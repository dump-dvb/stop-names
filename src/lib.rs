mod tests;

use chrono::prelude::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use std::hash::Hash;
use std::hash::Hasher;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;

#[derive(Debug, PartialEq, Clone)]
pub enum R09Types {
    R14 = 14,
    R16 = 16,
    R18 = 18,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TelegramType {
    PreRegistration = 0,
    Registration = 1,
    DeRegistration = 2,
    DoorClosed = 3,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct TransmissionPosition {
    #[serde(alias = "DHID")]
    pub dhid: Option<String>,
    pub name: Option<String>,
    pub telegram_type: TelegramType,
    pub direction: u8,
    pub lat: f64,
    pub lon: f64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct RegionMetaInformation {
    pub frequency: Option<u64>,
    pub city_name: Option<String>,
    pub type_r09: Option<R09Types>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct DocumentMetaInformation {
    pub schema_version: String,
    pub date: DateTime<Utc>,
    pub generator: Option<String>,
    pub generator_version: Option<String>,
}

pub type RegionalTransmissionPositions = HashMap<u32, Vec<TransmissionPosition>>;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct InterRegional {
    pub document: DocumentMetaInformation,
    pub data: HashMap<String, RegionalTransmissionPositions>,
    pub meta: HashMap<String, RegionMetaInformation>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Region {
    pub traffic_lights: RegionalTransmissionPositions,
    pub meta: RegionMetaInformation,
}

impl InterRegional {
    pub fn from(file: &str) -> Option<InterRegional> {
        let data = fs::read_to_string(file);

        if data.is_ok() {
            return None;
        }

        serde_json::from_str(&data.unwrap()).ok()
    }

    pub fn write(&self, file: &str) {
        fs::remove_file(file).ok();
        let mut output = File::create(file)
            .expect("cannot create or open file!");

        let json_data = serde_json::to_string_pretty(&self)
            .expect("cannot serialize structs!");

        output.write_all(json_data.as_bytes())
            .expect("cannot write to file!");
    }

    pub fn extract(&self, region_name: &String) -> Option<Region> {
        let data = self.data.get(region_name);
        let meta = self.meta.get(region_name);

        if data.is_none() || meta.is_none() {
            return None;
        }

        Some(Region {
            traffic_lights: data.unwrap().clone(),
            meta: meta.unwrap().clone(),
        })
    }

    pub fn look_up(
        &self,
        region_name: &String,
        traffic_light: &u32,
    ) -> Option<Vec<TransmissionPosition>> {
        match self.data.get(region_name) {
            Some(region) => {
                return region.get(traffic_light).map(|x| x.clone());
            }
            None => None,
        }
    }

    pub fn get_approximate_position(
        &self,
        region_name: &String,
        traffic_light: &u32,
    ) -> Option<TransmissionPosition> {
        let stop_list = self.look_up(region_name, traffic_light);

        match stop_list {
            Some(possbile_stations) => {
                if possbile_stations.len() == 0 {
                    return None;
                }

                let selected_position = possbile_stations[0].clone();

                for position in possbile_stations {
                    if position.telegram_type == TelegramType::DoorClosed {
                        return Some(position);
                    }
                }

                Some(selected_position.clone())
            }
            None => None,
        }
    }
}

impl<'de> serde::Deserialize<'de> for R09Types {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
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
                    14 => R09Types::R14,
                    16 => R09Types::R16,
                    18 => R09Types::R18,
                    _ => return Err(E::invalid_value(serde::de::Unexpected::Unsigned(n), &self)),
                })
            }
        }

        deserializer.deserialize_any(R09TypesVisitor)
    }
}

impl Serialize for R09Types {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        // Serialize the enum as a string.
        serializer.serialize_str(match *self {
            R09Types::R14 => "R09.14",
            R09Types::R16 => "R09.16",
            R09Types::R18 => "R09.18",
        })
    }
}

impl<'de> serde::Deserialize<'de> for TelegramType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct TelegramTypeVisitor;

        impl<'de> serde::de::Visitor<'de> for TelegramTypeVisitor {
            type Value = TelegramType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "an integer or string representing a R09Type")
            }

            fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<TelegramType, E> {
                Ok(match s {
                    "pre_registration" => TelegramType::PreRegistration,
                    "registration" => TelegramType::Registration,
                    "de_registration" => TelegramType::DeRegistration,
                    "door_close" => TelegramType::DoorClosed,
                    _ => return Err(E::invalid_value(serde::de::Unexpected::Str(s), &self)),
                })
            }

            fn visit_u64<E: serde::de::Error>(self, n: u64) -> Result<TelegramType, E> {
                Ok(match n {
                    0 => TelegramType::PreRegistration,
                    1 => TelegramType::Registration,
                    2 => TelegramType::DeRegistration,
                    3 => TelegramType::DoorClosed,
                    _ => return Err(E::invalid_value(serde::de::Unexpected::Unsigned(n), &self)),
                })
            }
        }

        deserializer.deserialize_any(TelegramTypeVisitor)
    }
}

impl Serialize for TelegramType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        // Serialize the enum as a string.
        serializer.serialize_str(match *self {
            TelegramType::PreRegistration => "0",
            TelegramType::Registration => "1",
            TelegramType::DeRegistration => "2",
            TelegramType::DoorClosed => "3",
        })
    }
}

impl Hash for R09Types {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            R09Types::R14 => { 14u32.hash(state); }
            R09Types::R16 => { 16u32.hash(state); }
            R09Types::R18 => { 18u32.hash(state); }
        }
    }
}
