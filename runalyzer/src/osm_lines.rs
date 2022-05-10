use std::collections::HashMap;
use std::fs::File;
use serde::Deserialize;
use geo::{prelude::GeodesicDistance, Point};
use super::{Error, Line};

#[derive(Debug, Deserialize)]
struct OverpassJson {
    elements: Vec<Record>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize)]
pub struct Id(u64);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize)]
enum RecordType {
    #[serde(rename = "node")]
    Node,
    #[serde(rename = "way")]
    Way,
    #[serde(rename = "relation")]
    Relation,
}

#[derive(Debug, Clone, Deserialize)]
struct Record {
    #[serde(rename = "type")]
    record_type: RecordType,
    id: Id,
    // for nodes
    lat: Option<f64>,
    lon: Option<f64>,
    // for ways
    nodes: Option<Vec<Id>>,
    // for relations
    members: Option<Vec<RelationMember>>,
    tags: Option<HashMap<String, String>>,
}

impl Record {
    fn waypoint(&self) -> Option<Waypoint> {
        if self.record_type != RecordType::Node {
            return None;
        }
        Some(Waypoint {
            id: self.id,
            lat: self.lat?,
            lon: self.lon?,
        })
    }

    fn line_stop(&self) -> Option<LineStop> {
        if self.record_type != RecordType::Node {
            println!("stop is not a node: {:?}", self);
        }
        Some(LineStop {
            name: self.tags.as_ref()?.get("name")?.to_string(),
            lat: self.lat?,
            lon: self.lon?,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize)]
struct RelationMember {
    #[serde(rename = "type")]
    record_type: RecordType,
    #[serde(rename = "ref")]
    record_ref: Id,
    role: String,
}

#[derive(Debug, Clone)]
pub struct LineInfo {
    pub line: Line,
    pub name: String,
    // pub stops: Vec<LineStop>,
    pub ways: Vec<Vec<Waypoint>>,
}

#[derive(Debug, Clone)]
pub struct Waypoint {
    pub id: Id,
    pub lat: f64,
    pub lon: f64,
}

impl PartialEq for Waypoint {
    fn eq(&self, rhs: &Self) -> bool {
        self.id == rhs.id
    }
}

impl Eq for Waypoint {}

impl LineInfo {
    fn detect_discontiguity(&self) -> bool {
        let mut discontiguities = 0;
        for i in 1..self.ways.len() {
            if !self.ways[i - 1].is_empty() &&
                !self.ways[i].is_empty() &&
                self.ways[i - 1][self.ways[i - 1].len() - 1] != self.ways[i][0] &&
                self.ways[i][self.ways[i].len() - 1] != self.ways[i - 1][0]
            {
                discontiguities += 1;
            }
        }
        if discontiguities > 0 {
            eprintln!("Line {}: {:?} contains {} discontiguities", self.line.0, self.name, discontiguities);
            true
        } else {
            false
        }
    }

    pub fn reorder_ways(&mut self) {
        let mut all_ways = vec![];
        while !self.ways.is_empty() {
            let mut ways = self.ways.remove(0);

            let mut done = false;
            while !done {
                done = true;

                self.ways.retain(|way| {
                    if way[0].id == ways[ways.len() - 1].id {
                        ways.extend_from_slice(way);
                        done = false;
                        false
                    } else if way[way.len() - 1].id == ways[0].id {
                        ways = way.iter().chain(ways.iter()).cloned().collect();
                        done = false;
                        false
                    } else if way[0].id == ways[0].id {
                        ways = way.iter().rev().chain(ways.iter()).cloned().collect();
                        done = false;
                        false
                    } else if way[way.len() - 1].id == ways[ways.len() - 1].id {
                        for waypoint in way.iter().rev() {
                            ways.push(waypoint.clone());
                        }
                        done = false;
                        false
                    } else {
                        true
                    }
                });
            }
            all_ways.push(ways);
        }
        self.ways = all_ways;
    }

    pub fn glue_ways(&mut self) {
        let mut all_ways = vec![];
        while !self.ways.is_empty() {
            let mut ways = self.ways.remove(0);

            let mut done = false;
            while !done {
                done = true;

                self.ways.retain(|way| {
                    fn dist(n: usize, w1: &Waypoint, w2: &Waypoint) -> (f64, usize) {
                        let distance = Point::new(w1.lon, w1.lat)
                            .geodesic_distance(&Point::new(w2.lon, w2.lat));
                        (distance, n)
                    }
                    let min_dist = [
                        dist(0, &way[0], &ways[ways.len() - 1]),
                        dist(1, &way[way.len() - 1], &ways[0]),
                        dist(2, &way[0], &ways[0]),
                        dist(3, &way[way.len() - 1], &ways[ways.len() - 1]),
                    ].iter().min_by(|(d1, _), (d2, _)| {
                        use std::cmp::Ordering;
                        if d1 < d2 {
                            Ordering::Less
                        } else if d1 > d2 {
                            Ordering::Greater
                        } else {
                            Ordering::Equal
                        }
                    }).map(|(_, n)| *n)
                        .unwrap();
                    match min_dist {
                        0 => {
                            ways.extend_from_slice(way);
                            done = false;
                            false
                        }
                        1 => {
                            ways = way.iter().chain(ways.iter()).cloned().collect();
                            done = false;
                            false
                        }
                        2 => {
                            ways = way.iter().rev().chain(ways.iter()).cloned().collect();
                            done = false;
                            false
                        }
                        3 => {
                            for waypoint in way.iter().rev() {
                                ways.push(waypoint.clone());
                            }
                            done = false;
                            false
                        }
                        _ => unreachable!(),
                    }
                });
            }
            all_ways.push(ways);
        }
        self.ways = all_ways;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineStop {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
}

pub fn read(path: &str) -> Result<Vec<LineInfo>, Box<dyn Error>> {
    println!("reading osm export {}", path);
    let mut infos = vec![];

    let file = File::open(path)?;
    let json: OverpassJson = serde_json::from_reader(file)?;
    println!("{} osm primitives", json.elements.len());

    let mut records = HashMap::new();
    for record in &json.elements {
        let r = records.entry((record.record_type, record.id))
            .or_insert(record);
        // some records are more than one time in the json file, only
        // sometimes with tags
        if r.tags.is_none() && record.tags.is_some() {
            *r = record;
        }
    }
    println!("{} records", records.len());

    for record in &json.elements {
        if record.record_type == RecordType::Relation {
            if let (Some(_members), Some(tags)) = (&record.members, &record.tags) {
                if [Some("route")].contains(&tags.get("type").map(std::string::String::as_str))
                && [Some("tram"), Some("bus")].contains(&tags.get("route").map(std::string::String::as_str)) {
                    let line = str::parse(tags.get("ref").expect("line ref"))
                        .ok()
                        .map(Line);
                    if let Some(line) = line {
                        let name = tags.get("name").map_or_else(|| format!("Linie {}", line.0), std::string::ToString::to_string);
                        let stops = if let Some(members) = &record.members {
                            members.iter().filter(|member| member.role == "stop")
                                .filter_map(|member| {
                                    assert_eq!(
                                        records.get(&(member.record_type, member.record_ref))
                                            .and_then(|record| record.line_stop()),
                                        json.elements.iter()
                                            .find(|record|
                                                  record.record_type == member.record_type &&
                                                  record.id == member.record_ref
                                            ).and_then(Record::line_stop)
                                    );
                                    let result = records.get(&(member.record_type, member.record_ref))
                                        .and_then(|record| record.line_stop());
                                    if result.is_none() {
                                        println!("Did not find stop with Id {}", member.record_ref.0);
                                    }
                                    result
                                }).collect()
                        } else {
                            vec![]
                        };
                        dbg!(stops.len());
                        let from_stop = tags.get("from")
                            .and_then(|from|
                                stops.iter()
                                    .map(|line_stop| (strsim::levenshtein(from, &line_stop.name), line_stop))
                                    .min_by_key(|(diff, _)| *diff)
                                    .map(|(_, line_stop)| line_stop)
                            );
                        let to_stop = tags.get("to")
                            .and_then(|to|
                                stops.iter()
                                    .find(|line_stop| line_stop.name == *to)
                            );
                        dbg!(from_stop, to_stop);
                        let ways = if let Some(members) = &record.members {
                            members.iter().filter(|member| member.record_type == RecordType::Way && member.role.is_empty())
                                .map(|member| {
                                    records.get(&(member.record_type, member.record_ref))
                                        .expect("way")
                                })
                                .map(|way| {
                                    way.nodes.as_ref().expect("way.nodes")
                                        .iter().map(|id| {
                                            records.get(&(RecordType::Node, *id))
                                                .and_then(|record| record.waypoint())
                                                .expect("way node")
                                        }).collect()
                                })
                                .collect()
                        } else {
                            vec![]
                        };
                        let mut info = LineInfo {
                            line,
                            name,
                            // stops,
                            ways,
                        };
                        info.reorder_ways();
                        if info.detect_discontiguity() {
                            println!("gluing");
                            info.glue_ways();
                            info.detect_discontiguity();
                        }

                        for ways in &mut info.ways {
                            let head_to_from = distance(&ways[0], from_stop);
                            let tail_to_from = distance(&ways[ways.len() - 1], from_stop);
                            let head_to_to = distance(&ways[0], to_stop);
                            let tail_to_to = distance(&ways[ways.len() - 1], to_stop);
                            match (head_to_from, head_to_to, tail_to_from, tail_to_to) {
                                (Some(head_to_from), _, Some(tail_to_from), _) if head_to_from > 10.0 * tail_to_from => {
                                    println!("Reversing ({:.0}m > {:.0}m)", head_to_from, tail_to_from);
                                    ways.reverse();
                                }
                                (_, Some(tail_to_from), _, Some(tail_to_to)) if 10.0 * tail_to_from < tail_to_to => {
                                    println!("Reversing ({:.0}m < {:.0}m)", tail_to_from, tail_to_to);
                                    ways.reverse();
                                }
                                (Some(_), _, Some(_), _) |
                                (_, Some(_), _, Some(_)) => {}
                                _ => {
                                    println!("Trouble finding exits for {}", line.0);
                                }
                            }
                        }

                        infos.push(info);
                    }
                }
            }
        }
    }

    Ok(infos)
}

fn distance(way: &Waypoint, line_stop: Option<&LineStop>) -> Option<f64> {
    line_stop.map(|line_stop|
                  Point::new(way.lon, way.lat)
                  .geodesic_distance(&Point::new(line_stop.lon, line_stop.lat))
    )
}
