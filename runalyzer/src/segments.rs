use std::collections::HashSet;
use std::time::{Duration, SystemTime};
use geo::{Line, Point};
use super::osm_lines::Waypoint;
use super::*;

pub fn junctions_by_known_stops(
    known_stops: &HashSet<Junction>, 
    run_junctions: Vec<(LineRun, Vec<(SystemTime, Junction)>)>,
) -> Vec<(LineRun, Vec<Junction>, Vec<(SystemTime, Junction)>)> {
    let mut results = vec![];

    for (line_run, junctions) in run_junctions {
        let stops = junctions.iter()
            .map(|(_, junction)| junction)
            .filter(|junction| known_stops.contains(junction))
            .cloned()
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
            next.push((time.clone(), *junction));
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
            (sum / total, *junction)
        })
        .collect()
}

pub fn way_point(ways: &Vec<Vec<Waypoint>>, known_point: &Point<f64>) -> Option<(usize, Point<f64>)> {
    let mut index = 0;
    ways.iter()
        .filter_map(|way| {
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
                        Closest::Intersection(p) => {
                            Some((index, known_point.euclidean_distance(&p), p))
                        }
                        Closest::SinglePoint(p) => {
                            Some((index, known_point.euclidean_distance(&p), p))
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
                    // within 35m
                    if distance < 0.0005 {
                        Some((index, closest_point))
                    } else {
                        None
                    }
                }).map(|(index, closest_point)| {
                    (index, closest_point)
                })
        }).next()
}

#[derive(Clone, Debug)]
pub struct Segment {
    pub start: (Junction, Point<f64>),
    pub stop: (Junction, Point<f64>),
    pub segment: Vec<(f64, Junction)>,
}

pub enum ResultSegment {
    Junction(Junction, Point<f64>),
    Point(Point<f64>),
}

// segments must be ordered
pub fn segmentize(
    segment: &Segment,
    ways: &Vec<Vec<Waypoint>>,
) -> Vec<ResultSegment> {
    let mut result = vec![];

    result.push(ResultSegment::Junction(segment.start.0, segment.start.1));


    
    result.push(ResultSegment::Junction(segment.stop.0, segment.stop.1));

    result
}
