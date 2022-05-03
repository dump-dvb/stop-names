use std::collections::HashMap;
use std::error::Error;
use std::time::{Duration, SystemTime};
use serde::Deserialize;

const JUNCTION_MAX_DURATION: u64 = 20 * 60;

mod osm_lines;

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

#[derive(Debug, Clone, Deserialize)]
struct Telegram {
    time_stamp: u64,
    // lat: f64,
    // lon: f64,
    // station_id: u64,
    line: Line,
    // destination_number: u64,
    // priority: (),
    // sign_of_deviation: (),
    // value_of_deviation: (),
    // reporting_point: (),
    // request_for_priority: (),
    run_number: Run,
    // reserve: (),
    // train_length: (),
    junction: Junction,
    // junction_number: u16,
}

fn read_telegrams(path: &str) -> Result<HashMap<LineRun, Vec<(SystemTime, Junction)>>, Box<dyn Error>> {
    let mut by_run: HashMap<LineRun, Vec<(SystemTime, Junction)>> = HashMap::new();
    for result in csv::Reader::from_path(path)?.deserialize::<Telegram>() {
        match result {
            Err(e) => {
                eprintln!("Parse error: {}", e);
            }
            Ok(telegram) => {
                let line_run = LineRun { line: telegram.line, run: telegram.run_number };
                let time = SystemTime::UNIX_EPOCH + Duration::from_secs(telegram.time_stamp);
                by_run.entry(line_run)
                    .or_default()
                    .push((time, telegram.junction));
            }
        }
    }

    Ok(by_run)
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut stamps_by_run = read_telegrams("../../old1.csv")?;
    for stamps in stamps_by_run.values_mut() {
        stamps.sort();
    }

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

    let lines = osm_lines::read("trams.json")?;
    // dbg!(lines);
    
    Ok(())
}
