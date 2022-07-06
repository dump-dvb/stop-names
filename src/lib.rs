use std::collections::HashMap;
use serde::{
    Serialize, 
    Deserialize,
};

use std::fs;

#[derive(Debug, PartialEq, Clone)]
pub enum R09Types {
    R14 = 14,
    R16 = 16,
    R18 = 18
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Stop {
    #[serde(alias = "DHID")] 
    dhid: Option<String>,
    name: Option<String>,
    telegram_type: R09Types,
    direction: u8,
    lan: f64,
    lon: f64
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


pub fn parse_from_file(file: &str) -> Stops {
    let data = fs::read_to_string(file).expect("Unable to read file");
    let res: Stops = serde_json::from_str(&data).expect("Unable to parse");
    return res;
}

impl<'de> serde::Deserialize<'de> for R09Types {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        struct R09TypesVisitor;

        impl<'de> serde::de::Visitor<'de> for R09TypesVisitor {
            type Value = R09Types;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "an integer or string representing a R09Type")
            }

            fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<R09Types, E> {
                Ok(match s {
                    "R09.14" => R09Types::R14,
                    "R09.16" => R09Types::R16,
                    "R09.18" => R09Types::R18,
                    _ => return Err(E::invalid_value(serde::de::Unexpected::Str(s), &self)),
                })
            }

            fn visit_u64<E: serde::de::Error>(self, n: u64) -> Result<R09Types, E> {
                Ok(match n {
                    1 => R09Types::R14,
                    2 => R09Types::R16,
                    3 => R09Types::R18,
                    _ => return Err(E::invalid_value(serde::de::Unexpected::Unsigned(n), &self)),
                })
            }
        }

        deserializer.deserialize_any(R09TypesVisitor)
    }
}

impl Serialize for R09Types {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: ::serde::Serializer,
        {
            // Serialize the enum as a string.
            serializer.serialize_str(match *self {
                R09Types::R14 => "R09.14",
                R09Types::R16 => "R09.16",
                R09Types::R18 => "R09.18",
            })
        }
}

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

