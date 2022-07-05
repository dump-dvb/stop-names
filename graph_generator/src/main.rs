#![feature(drain_filter)]

pub mod telegram;


use telegram::{read_telegrams, Junction, Direction};
use std::collections::HashMap;
use serde_json;
use std::fs::File;
use std::io::Write;

fn index_of_max(values: &[u32]) -> Option<usize> {
    values
        .iter()
        .enumerate()
        .max_by_key(|(_idx, &val)| val)
        .map(|(idx, _val)| idx)
}


fn choose_junction(vec: Vec<Junction>) -> Option<Junction> {
    let mut occurences: HashMap<Junction, u32> = HashMap::new();
    for element in vec {
        if occurences.contains_key(&element) {
            *occurences.get_mut(&element).unwrap() += 1;
        } else {
            occurences.insert(element, 1);
        }
    }

    let values: Vec<u32> = occurences.clone().into_values().collect();
    let keys: Vec<Junction> = occurences.into_keys().collect();

    match index_of_max(values.as_slice()) {
        Some(index) => {
            Some(keys[index])
        }
        None => {
            None
        }
    }
}

fn main() {
    println!("Starting Script ... ");

    let path: String = String::from("./formatted.csv");
    let telegrams = read_telegrams(&path).unwrap();
    let mut graph: HashMap<Junction, HashMap<Direction, Junction>> = HashMap::new();
    let mut measured: HashMap<(Junction, Direction), Vec<Junction>> = HashMap::new();
    let time_limit_future: u64 = 300;

    for i in 0..telegrams.len() - 1 {
        let current_tele = &telegrams[i];

        let mut time = current_tele.time;
        let mut iterator = i + 10;
        while (time.timestamp() as u64) < current_tele.time.timestamp() as u64 + time_limit_future &&  iterator < telegrams.len() {
            let future_tele = &telegrams[iterator];
            time = future_tele.time;

            if current_tele.line == future_tele.line && current_tele.run_number == future_tele.run_number {
                match measured.get_mut(&(current_tele.junction, current_tele.direction_request)) {
                    Some(value) => {
                        value.push(future_tele.junction);
                    }
                    None => {
                        measured.insert((current_tele.junction, current_tele.direction_request), vec![future_tele.junction]);
                    }
                }
            }

            iterator += 1
        }
    }

    for (key, mut value) in measured {
        let drained: Vec<Junction> = value.drain_filter(|&mut e| e != key.0).collect();
        match choose_junction(drained) {
            Some(target_junction) => {
                match graph.get_mut(&key.0) {
                    Some(data) => {
                        data.insert(key.1, target_junction);
                    }
                    None => {
                        graph.insert(key.0, HashMap::from([(key.1, target_junction)]));
                    }
                }
            }
            None => {}
        };
    }
    let j = serde_json::to_string_pretty(&graph).unwrap();
    let mut output = File::create("./out.json").unwrap();
    output.write_all(j.as_bytes()).unwrap();
}

