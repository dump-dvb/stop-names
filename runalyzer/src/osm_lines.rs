use std::fs::File;
use serde::Deserialize;
use super::*;

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

// impl Record {
//     fn line_stop(&self) -> Option<LineStop> {
//         if self.record_type != RecordType::Node {
//             println!("stop is not a node: {:?}", self);
//         }
//         Some(LineStop {
//             name: self.tags.as_ref()?.get("name")?.to_string(),
//             lat: self.lat?,
//             lon: self.lon?,
//         })
//     }
// }

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
    fn detect_discontiguity(&self) {
        let mut discontiguities = 0;
        for i in 1..self.ways.len() {
            if self.ways[i - 1].len() > 0 &&
                self.ways[i].len() > 0 &&
                self.ways[i - 1][self.ways[i - 1].len() - 1] != self.ways[i][0] &&
                self.ways[i][self.ways[i].len() - 1] != self.ways[i - 1][0]
            {
                discontiguities += 1;
            }
        }
        if discontiguities > 0 {
            eprintln!("Line {}: {:?} contains {} discontiguities", self.line.0, self.name, discontiguities);
        }
    }

    pub fn reorder_ways(&mut self) {
        let mut all_ways = vec![];
        while self.ways.len() > 0 {
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
}

#[derive(Debug, Clone)]
pub struct LineStop {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
}

pub fn read(path: &str) -> Result<Vec<LineInfo>, Box<dyn Error>> {
    let mut infos = vec![];

    let file = File::open(path)?;
    let json: OverpassJson = serde_json::from_reader(file)?;

    let records = json.elements.iter()
        .map(|record| ((record.record_type, record.id), record))
        .collect::<HashMap<_, _>>();
    
    for record in &json.elements {
        match record.record_type {
            RecordType::Relation => {
                if let (Some(members), Some(tags)) = (&record.members, &record.tags) {
                    if [Some("route")].contains(&tags.get("type").map(|s| s.as_str())) && [Some("tram"), Some("bus")].contains(&tags.get("route").map(|s| s.as_str())) {
                        let line = str::parse(tags.get("ref").expect("line ref"))
                            .ok()
                            .map(Line);
                        // let stops = if let Some(members) = &record.members {
                        //     members.iter().filter(|member| member.role == "stop")
                        //         .filter_map(|member| {
                        //             records.get(&(member.record_type, member.record_ref))
                        //                 .and_then(|record| record.line_stop())
                        //         }).collect()
                        // } else {
                        //     vec![]
                        // };
                        let ways = if let Some(members) = &record.members {
                            members.iter().filter(|member| member.record_type == RecordType::Way && member.role == "")
                                .map(|member| {
                                    records.get(&(member.record_type, member.record_ref))
                                        .expect("way")
                                })
                                .map(|way| {
                                    way.nodes.as_ref().expect("way.nodes")
                                        .iter().map(|id| {
                                            records.get(&(RecordType::Node, *id))
                                                .map(|record| Waypoint {
                                                    id: *id,
                                                    lat: record.lat.expect("lat"),
                                                    lon: record.lon.expect("lon"),
                                                })
                                                .expect("way node")
                                        }).collect()
                                })
                                .collect()
                        } else {
                            vec![]
                        };
                        if let Some(line) = line {
                            let mut info = LineInfo {
                                line,
                                name: tags.get("name").map(|s| s.to_string())
                                    .unwrap_or_else(|| format!("Linie {}", line.0)),
                                // stops,
                                ways,
                            };
                            info.reorder_ways();
                            info.detect_discontiguity();
                            infos.push(info);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok(infos)
}
