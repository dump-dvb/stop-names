use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::time::{Duration, SystemTime};
use serde::Deserialize;
use geo::{coord, LineString, prelude::{ClosestPoint, EuclideanDistance}, Closest, Point};

const JUNCTION_MAX_DURATION: u64 = 20 * 60;

mod telegram;
mod osm_lines;
mod known_stops;
mod segments;

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

pub struct Segment {
    junctions: Vec<(Junction, Option<Duration>)>,
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("loading known stops");
    let stops = known_stops::load("../stops.json")?;
    let known_stops = stops.keys().cloned().collect::<HashSet<_>>();
    println!("{} stops loaded", stops.len());
    
    println!("reading telegrams");
    let run_junctions = telegram::read_telegrams("../../formatted.csv")?;
    let junctions_by_known_stops = segments::junctions_by_known_stops(
        &known_stops,
        run_junctions
    );

    let mut lines = HashMap::<Line, Vec<osm_lines::LineInfo>>::new();
    for line_info in osm_lines::read("../trams.json")?.into_iter()
        .chain(osm_lines::read("../buses.json")?.into_iter())
    {
        lines.entry(line_info.line)
            .or_default()
            .push(line_info);
    }

    for (line, line_infos) in lines.into_iter() {
        for line_info in line_infos {
            let mut line_known_stops = stops.iter().filter_map(|(junction, stop)| {
                let known_point = Point::new(stop.lon, stop.lat);
                segments::way_point(&line_info.ways, &known_point)
                    .map(|(index, point)| (index, *junction, point))
            }).collect::<Vec<_>>();
            line_known_stops.sort_by_key(|(index, _, _)| *index);
            println!("Found {} known stops in OSM {}", line_known_stops.len(), line_info.name);
            if line_known_stops.len() < 2 {
                continue;
            }

            let known_stop_junctions = line_known_stops.iter()
                .map(|(_, junction, _)| junction)
                .cloned()
                .collect::<Vec<Junction>>();
            let mut best_length = 0;
            let mut matching_runs = vec![];
            for (line_run, known_junctions, junctions) in &junctions_by_known_stops {
                if line_run.line != line {
                    continue;
                }

                if known_junctions.len() > 1
                && is_similar_sequence(known_junctions, &known_stop_junctions) {

                    if known_stop_junctions.len() > best_length {
                        best_length = known_stop_junctions.len();
                        matching_runs = vec![];
                    }

                    matching_runs.push(junctions);
                }
            }
            println!("telegrams contain {} good matching runs", matching_runs.len());
            // longest known junctions segment between known stations
            let mut longest_segments = HashMap::new();
            let mut min_durations = HashMap::new();
            for junctions in matching_runs.into_iter() {
                // best junction path between stations
                for ((start, stop), segment) in segments::segment_run_by_known_stops(&known_stops, junctions) {
                    let mut last_junction = None;
                    for (duration, junction) in segment.clone() {
                        if let Some(last_junction) = last_junction.take() {
                            let min_duration = min_durations.entry((last_junction, junction))
                                .or_insert(duration);
                            if duration < *min_duration {
                                *min_duration = duration;
                            }
                        }

                        last_junction = Some(junction);
                    }

                    let longest_segment = longest_segments.entry((start, stop))
                        .or_insert_with(|| segment.clone());
                    if longest_segment.len() < segment.len() {
                        *longest_segment = segment.clone();
                    }
                }

            }
            // apply min_durations with each longest_segment
            let longest_segments = longest_segments.into_iter()
                .map(|((start, stop), segment)| {
                    let mut last_junction = None;
                    let mut min_segment = Vec::with_capacity(segment.len());
                    for (duration, junction) in segment.into_iter() {
                        let result = if let Some(last_junction) = last_junction {
                            if let Some(min_duration) = min_durations.get(&(last_junction, junction)) {
                                (*min_duration, junction)
                            } else {
                                println!("No best duration for {:?}", (last_junction, junction));
                                (duration, junction)
                            }
                        } else {
                                (duration, junction)
                        };
                        min_segment.push(result);

                        last_junction = Some(junction);
                    }
                    ((start, stop), min_segment)
                })
                .collect::<HashMap<_, _>>();

            let mut last = None;
            let known_stop_segments = line_known_stops.into_iter()
                .filter_map(|(_, junction, point)| {
                    let result = if let Some((last_junction, last_point)) = last.take() {
                        if let Some(longest_segment) = longest_segments.get(&(last_junction, junction)) {
                            let segment = segments::to_rational(longest_segment);
                            Some(segments::Segment {
                                start: (last_junction, last_point),
                                stop: (junction, point),
                                segment,
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    last = Some((junction, point));
                    result
                }).collect::<Vec<_>>();
            dbg!(known_stop_segments);
        }
    }
    
    Ok(())
}

fn is_similar_sequence(partial: &[Junction], goal: &[Junction]) -> bool {
    let partial_set = partial.iter().collect::<HashSet<_>>();
    let partial_of_goal = goal.iter()
        .filter(|g| partial_set.contains(g))
        .cloned()
        .collect::<Vec<_>>();
    &partial_of_goal[..] == partial
}
