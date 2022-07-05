use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Stop {
    telegram_type: u8,
    direction: u8,
    lan: f64,
    lon: f64
}

#[derive(Serialize, Deserialize)]
pub struct Stops(HashMap<u64, HashMap<u32, Vec<Stop>>>);

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

