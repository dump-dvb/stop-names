use std::error::Error;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct Line(u16);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct Run(u16);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct Junction(u32);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct Direction(u32);


#[derive(Debug, Clone, Deserialize)]
pub struct Telegram {
    pub time: DateTime<Utc>,
    pub ip: String,
    pub line: Line,
    // destination_number: u64,
    // priority: (),
    // sign_of_deviation: (),
    // value_of_deviation: (),
    // reporting_point: (),
    pub direction_request: Direction,
    pub run_number: Run,
    // reserve: (),
    // train_length: (),
    pub junction: Junction,
    // junction_number: u16,
}

pub fn read_telegrams(path: &str) -> Result<Vec<Telegram>, Box<dyn Error>> {
    let mut amount = 0;
    let mut errors = 0;
    let mut results = vec![];

    let desired_station = vec![String::from("10.13.37.100"), String::from("10.13.37.101")];

    for result in csv::Reader::from_path(path)?.deserialize::<Telegram>() {
        match result {
            Err(e) => {
                eprintln!("Parse error: {}", e);
                errors += 1;
            }
            Ok(telegram) => {
                if desired_station.contains(&telegram.ip) {
                    results.push(telegram);
                    amount += 1
                }
            }
        }
    }

    println!("{}: parsed {} telegrams into {} line runs, {} errors", path, amount, results.len(), errors);
    Ok(results)
}
