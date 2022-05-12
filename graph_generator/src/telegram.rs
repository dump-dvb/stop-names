use std::error::Error;
use serde::{Deserialize, Serialize};

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
    pub time_stamp: u64,
    // lat: f64,
    // lon: f64,
    // station_id: u64,
    pub line: Line,
    // destination_number: u64,
    // priority: (),
    // sign_of_deviation: (),
    // value_of_deviation: (),
    // reporting_point: (),
    pub request_for_priority: Direction,
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

    for result in csv::Reader::from_path(path)?.deserialize::<Telegram>() {
        match result {
            Err(e) => {
                eprintln!("Parse error: {}", e);
                errors += 1;
            }
            Ok(telegram) => {
                results.push(telegram);
                amount += 1
            }
        }
    }

    println!("{}: parsed {} telegrams into {} line runs, {} errors", path, amount, results.len(), errors);
    Ok(results)
}
