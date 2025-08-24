use std::{
    fs::File,
    io::Write,
    path::{self, PathBuf},
};

use crate::maps::parser::{MapSerializer, SSPMSerializer};

mod io;
mod maps;

fn main() {
    let path = path::PathBuf::from("/home/faded/Downloads/kocmoc.sspm");
    let map = SSPMSerializer::deserialize(path).unwrap();
    println!("{}", map.difficulty_name);
    File::create("/home/faded/Downloads/audio.mp3")
        .unwrap()
        .write_all(&map.audio)
        .unwrap();
    SSPMSerializer::serialize(
        &map,
        PathBuf::from("/home/faded/Downloads/kocmoc_output.sspm"),
    )
    .unwrap();
}
