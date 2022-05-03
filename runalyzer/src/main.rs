use std::collections::HashMap;
use std::error::Error;
use std::time::SystemTime;
use serde::Deserialize;

const JUNCTION_MAX_DURATION: u64 = 20 * 60;
const STOP_MAX_DIFF: usize = 5;

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
    let get_stop = |s| {
        if let Some(stop) = stops.get(s) {
            Some(stop)
        } else if Some(stop) = stops.iter().min_by_key(|stop| levenshtein(s, stop.name)) {
            if levenshtein(s, stop.name) <= STOP_MAX_DIFF {
                Some(stop)
            } else {
                None
            }
        }
    };
    
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
        let mut junctions = [];
        for line_info in line_infos {
            // find known stops
        }
        line_junctions.push(junctions);
    }
    dbg!(line_junctions);
    
    Ok(())
}
