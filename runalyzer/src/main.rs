use std::collections::HashMap;
use std::error::Error;
use std::time::SystemTime;
use serde::Deserialize;
use geo::{coord, LineString, prelude::{ClosestPoint, EuclideanDistance}, Closest, Point};

const JUNCTION_MAX_DURATION: u64 = 20 * 60;

mod telegram;
mod osm_lines;
mod known_stops;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct LineRun {
    line: Line,
    run: Run,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize)]
pub struct Line(u16);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize)]
pub struct Run(u16);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize)]
pub struct Junction(u32);

fn main() -> Result<(), Box<dyn Error>> {
    println!("loading known stops");
    let stops = known_stops::load("../stops.json")?;
    println!("{} stops loaded", stops.len());
    
    println!("reading telegrams");
    let mut stamps_by_run = telegram::read_telegrams("../../old1.csv")?;
    println!("sorting {} telegrams", stamps_by_run.len());
    for stamps in stamps_by_run.values_mut() {
        stamps.sort();
    }

    println!("collecting minimal durations");
    let durations_by_run = stamps_by_run.into_iter()
        .map(|(line_run, stamps)| {
            let mut last_stamp: Option<(SystemTime, Junction)> = None;
            let mut durations = HashMap::new();
            for stamp@(time, junction) in stamps.into_iter() {
                if let Some((last_time, last_junction)) = last_stamp.take() {
                    if last_junction != junction {
                        let duration = time.duration_since(last_time)
                            .expect("duration_since");
                        if duration.as_secs() > JUNCTION_MAX_DURATION {
                            continue;
                        }
                        let min_duration = durations.entry((last_junction, junction))
                            .or_insert(duration);
                        if duration < *min_duration {
                            *min_duration = duration;
                        }
                    }
                }

                last_stamp = Some(stamp);
            }

            (line_run, durations)
        })
        .collect::<HashMap<_, _>>();
    // dbg!(durations_by_run.keys().map(|lr|lr.line).collect::<Vec<_>>());

    println!("processing osm data");
    let mut lines = HashMap::<Line, Vec<osm_lines::LineInfo>>::new();
    for line_info in osm_lines::read("trams.json")?.into_iter()
        .chain(osm_lines::read("buses.json")?.into_iter())
    {
        lines.entry(line_info.line)
            .or_default()
            .push(line_info);
    }

    let mut line_junctions = vec![];
    for (line, line_infos) in lines.into_iter() {
        println!("tracing junctions of line {}", line.0);
        let mut junctions = vec![()];
        for line_info in line_infos {
            let known_stops = stops.iter().filter_map(|(junction, stop)| {
                dbg!(&junction);
                let known_point = Point::new(stop.lon, stop.lat);
                dbg!(line_info.ways.len());
                line_info.ways.iter()
                    .filter_map(|way| {
                        let line = LineString::new(
                            way.iter().map(|waypoint| coord! {
                                x: waypoint.lon,
                                y: waypoint.lat,
                            }).collect()
                        );
                        match line.closest_point(&known_point) {
                            Closest::Intersection(p) => {
                                Some((known_point.euclidean_distance(&p), p, line))
                            }
                            Closest::SinglePoint(p) => {
                                Some((known_point.euclidean_distance(&p), p, line))
                            }
                            Closest::Indeterminate => None,
                        }
                    }).min_by(|(d1, _, _), (d2, _, _)| {
                        use std::cmp::Ordering;
                        if d1 < d2 {
                            Ordering::Less
                        } else if d1 > d2 {
                            Ordering::Greater
                        } else {
                            Ordering::Equal
                        }
                    })
                    .and_then(|closest_point| {
                        if closest_point.0 < 0.01 {
                            Some(closest_point)
                        } else {
                            None
                        }
                    }).map(|closest_point| {
                        (junction, closest_point.1)
                    })
            }).collect::<HashMap<_, _>>();
            dbg!(known_stops);
        }
        line_junctions.push((line, junctions));
    }
    // dbg!(line_junctions);
    
    Ok(())
}
