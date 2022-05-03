use std::collections::HashMap;
use std::error::Error;
use std::time::{Duration, SystemTime};
use serde::Deserialize;
use super::*;

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

pub fn read_telegrams(path: &str) -> Result<HashMap<LineRun, Vec<(SystemTime, Junction)>>, Box<dyn Error>> {
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
