use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Stop {
    #[serde(alias = "DHID")] 
    dhid: Option<String>,
    name: Option<String>,
    telegram_type: u8,
    direction: u8,
    lan: f64,
    lon: f64
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum R09Types {
    R14,
    R16,
    R18
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct RegionMetaInformation {
    frequency: Option<u64>,
    city_name: Option<String>,
    type_r09: Option<R09Types>
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Region {
    #[serde(flatten)]
    stops: HashMap<u32, Vec<Stop>>,

    meta: RegionMetaInformation
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Stops(HashMap<u64, Region>);

/*
{

    "14625020": { // ZHV (aka Germany-wide) gemeindeschlüssel
        "1234": [ // traffic light ID
            {
                "telegram_type": 0, // possible values: pre_registration (0), registration (1), de_registration (2), doors_close (3)
                "direction": 1, // direction identifier
                "lat": 54.88141,
                "lon": 8.291386
            },
            {
                "telegram_type": 3, // doors close actually means we are most probably at a stop
                "direction": 1,
                "lat": 54.881241,
                "lon": 8.29131,
                "DHID": "de:01001:104053", // OPTIONAL Stop ID (ZHV, Germany-wide)
                "name": "Katzenstr." // OPTIONAL
            }
        ],
        "meta": {
            "frequency": 105200000,
            "city_name": "Westerland",
            "telegram_format": "R09.18"
        }
    }
}
*/
