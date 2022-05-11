use std::fs::File;
use serde::Deserialize;
use super::{Error, HashMap, Junction};

#[derive(Debug, Deserialize)]
pub struct Stop {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
}

pub fn load(path: &str) -> Result<HashMap<Junction, Stop>, Box<dyn Error>> {
    let file = File::open(path)?;
    let stops = serde_json::from_reader(file)?;
    Ok(stops)
}
