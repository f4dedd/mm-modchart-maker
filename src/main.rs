use std::path;

use crate::maps::parser::{MapSerializer, SSPMParser};

mod io;
mod maps;

fn main() {
    let path = path::PathBuf::from("/home/faded/Downloads/kocmoc.sspm");
    let map = SSPMParser::deserialize(path).unwrap();

    println!("{}", map.title);
}
