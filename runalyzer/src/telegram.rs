use std::collections::HashMap;
use std::error::Error;
use std::time::{Duration, SystemTime};
use serde::Deserialize;
use super::{Junction, Line, LineRun, Run};

const RUN_MAX_GAP: Duration = Duration::from_secs(1800);

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

pub fn read_telegrams(path: &str) -> Result<Vec<(LineRun, Vec<(SystemTime, Junction)>)>, Box<dyn Error>> {
    let mut amount = 0;
    let mut errors = 0;
    let mut results = vec![];
    let mut current = HashMap::<LineRun, Vec<(SystemTime, Junction)>>::new();

    for result in csv::Reader::from_path(path)?.deserialize::<Telegram>() {
        match result {
            Err(e) => {
                eprintln!("Parse error: {}", e);
                errors += 1;
            }
            Ok(telegram) => {
                let line_run = LineRun { line: telegram.line, run: telegram.run_number };
                let time = SystemTime::UNIX_EPOCH + Duration::from_secs(telegram.time_stamp);
                let junctions = current.entry(line_run)
                    .or_default();
                if Some(telegram.junction) != junctions.last().map(|(_time, junction)| *junction) {
                    junctions.push((time, telegram.junction));
                }
                amount += 1;

                current.retain(|line_run, junctions| {
                    let last_update = junctions.last().unwrap().0;
                    if last_update + RUN_MAX_GAP < time {
                        results.push((*line_run, junctions.split_off(0)));
                        false
                    } else {
                        true
                    }
                });
            }
        }
    }

    for (line_run, junctions) in current {
        results.push((line_run, junctions));
    }

    println!("{}: parsed {} telegrams into {} line runs, {} errors", path, amount, results.len(), errors);

    Ok(results)
}
