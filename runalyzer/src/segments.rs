use std::collections::{HashMap, HashSet};
use std::time::{Duration, SystemTime};
use geo::{prelude::{EuclideanDistance, GeodesicDistance, GeodesicLength}, LineString, Point, coord};
use serde::{Serialize, Serializer, ser::{SerializeStruct, SerializeTuple}};
use super::osm_lines::Waypoint;
use super::{Closest, ClosestPoint, Junction, LineRun};

pub fn junctions_by_known_stops(
    known_stops: &HashSet<Junction>, 
    run_junctions: Vec<(LineRun, Vec<(SystemTime, Junction)>)>,
) -> Vec<(LineRun, Vec<Junction>, Vec<(SystemTime, Junction)>)> {
    let mut results = vec![];

    for (line_run, junctions) in run_junctions {
        let stops = junctions.iter()
            .map(|(_, junction)| junction)
            .filter(|junction| known_stops.contains(junction))
            .copied()
            .collect::<Vec<_>>();

        results.push((line_run, stops, junctions));
    }

    results
}

pub fn segment_run_by_known_stops(
    known_stops: &HashSet<Junction>,
    junctions: &[(SystemTime, Junction)],
) -> HashMap<(Junction, Junction), Vec<(Duration, Junction)>> {
    let mut by_known = HashMap::new();

    let mut last_known = None;
    let mut next = vec![];
    for (time, junction) in junctions {
        if last_known.is_some() {
            next.push((*time, *junction));
        }

        if known_stops.contains(junction) {
            if let Some(last_known) = last_known.take() {
                let current = std::mem::replace(&mut next, vec![(*time, *junction)]);
                let mut last_time = None;
                let segment = current.into_iter()
                    .map(|(time, junction): (SystemTime, Junction)| {
                        let result = if let Some(last_time) = last_time.take() {
                            (time.duration_since(last_time).unwrap(), junction)
                        } else {
                            (Duration::ZERO, junction)
                        };
                        last_time = Some(time);
                        result
                    })
                    .collect::<Vec<_>>();
                by_known.insert((last_known, *junction), segment);
            }
            last_known = Some(*junction);
        }
    }

    by_known
}

pub fn to_rational(durations: &[(Duration, Junction)]) -> Vec<(f64, Junction)> {
    let total: f64 = durations.iter()
        .map(|(duration, _)| duration.as_secs_f64())
        .sum();
    let mut sum = 0.0;
    durations.iter()
        .map(|(duration, junction)| {
            let d = duration.as_secs_f64();
            sum += d;
            (if total == 0.0 { 0.0 } else { sum / total }, *junction)
        })
        .collect()
}

pub fn way_point(ways: &[Vec<Waypoint>], known_point: &Point<f64>) -> Option<(usize, Point<f64>)> {
    let mut index = 0;
    ways.iter()
        .find_map(|way| {
            let linestring = LineString::new(
                way.iter().map(|waypoint| coord! {
                    x: waypoint.lon,
                    y: waypoint.lat,
                }).collect()
            );
            linestring.lines()
                .filter_map(|line| {
                    index += 1;
                    match line.closest_point(known_point) {
                        Closest::Intersection(p) | Closest::SinglePoint(p) => {
                            Some((index, known_point.geodesic_distance(&p), p))
                        }
                        Closest::Indeterminate => None,
                    }
                }).min_by(|(_, d1, _), (_, d2, _)| {
                    use std::cmp::Ordering;
                    if d1 < d2 {
                        Ordering::Less
                    } else if d1 > d2 {
                        Ordering::Greater
                    } else {
                        Ordering::Equal
                    }
                })
                .and_then(|(index, distance, closest_point)| {
                    // meters
                    if distance < 30.0 {
                        Some((index, closest_point))
                    } else {
                        None
                    }
                }).map(|(index, closest_point)| {
                    (index, closest_point)
                })
        })
}

fn split_linestring_at_point(linestring: LineString<f64>, point: &Point<f64>) -> (LineString<f64>, LineString<f64>) {
    let mut min_dist = None;
    let mut line_index = None;
    for (index, line) in linestring.lines().enumerate() {
        let dist = line.euclidean_distance(&point.0);
        if min_dist.map_or(true, |min_dist| dist < min_dist) {
            min_dist = Some(dist);
            line_index = Some(index);
        }
    }

    if let Some(line_index) = line_index {
        let points = linestring.into_points();
        let (lines1, lines2) = points.split_at(line_index + 1);
        return (LineString::new(lines1.iter().map(|p| p.0).chain([point.0].into_iter()).collect()),
                LineString::new([point.0].into_iter().chain(lines2.iter().map(|p| p.0)).collect()));
    }

    (linestring, LineString::new(vec![]))
}

// TODO: use wgs84
fn linestring_length(linestring: &LineString<f64>) -> f64 {
    linestring.lines()
        .map(|line| line.geodesic_length())
        .sum()
}

#[derive(Clone, Debug)]
pub struct Segment {
    pub start: (Junction, Point<f64>),
    pub stop: (Junction, Point<f64>),
    pub junctions: Vec<(f64, Junction)>,
}

#[derive(Clone, Debug)]
pub enum ResultSegment {
    Junction(Junction, Point<f64>),
    Point(Point<f64>),
}

impl Serialize for ResultSegment {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ResultSegment::Point(p) => {
                let mut state = serializer.serialize_tuple(2)?;
                state.serialize_element(&p.x())?;
                state.serialize_element(&p.y())?;
                state.end()
            }
            ResultSegment::Junction(junction, p) => {
                let mut state = serializer.serialize_struct("Junction", 2)?;
                state.serialize_field("junction", &junction.0)?;
                state.serialize_field("pos", &[p.x(), p.y()])?;
                state.end()
            }
        }
    }
}

// segments must be ordered
pub fn segmentize(
    segment: &Segment,
    ways: &[Vec<Waypoint>],
) -> Vec<Vec<ResultSegment>> {
    if segment.junctions.is_empty() {
        return vec![];
    }
    
    ways.iter().filter_map(|way| {
        let linestring = LineString::new(
            way.iter().map(|waypoint| coord! {
                x: waypoint.lon,
                y: waypoint.lat,
            }).collect()
        );
        let (_, linestring) = split_linestring_at_point(linestring, &segment.start.1);
        let (linestring, _) = split_linestring_at_point(linestring, &segment.stop.1);
        // return early if empty
        linestring.lines().next()?;
        let length = linestring_length(&linestring);

        let mut results = vec![];
        let mut distance = 0.0;
        let mut junction_index = 0;
        for line in linestring.lines() {
            let new_distance = distance + line.geodesic_length();
            while junction_index < segment.junctions.len() {
                let junction = &segment.junctions[junction_index];
                let junction_distance = junction.0 * length;
                if new_distance > junction_distance {
                    dbg!(junction_distance);
                    let point = line.start_point() + (line.delta() * ((junction_distance - distance) / (new_distance - distance))).into();
                    results.push(ResultSegment::Junction(junction.1, point));
                    junction_index += 1;
                } else {
                    break
                }

            }

            results.push(ResultSegment::Point(line.end_point()));
            distance = new_distance;
        }

        if junction_index != segment.junctions.len() {
            println!("not all segments processed");
        }
        Some(results)
    }).collect()
}
