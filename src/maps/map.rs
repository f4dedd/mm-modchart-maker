use crate::maps::objects::note::Note;

use super::parser::ObjectDefinition;

pub trait MapInfo {}

pub trait MapMeta {
    fn get_title(&self) -> String;
    fn get_mappers(&self) -> Vec<String>;
    fn get_artists(&self) -> Vec<String>;
    fn get_length(&self) -> u32;
}

#[derive(Debug)]
pub struct Map {
    pub id: String,
    pub length: u32,
    pub title: String,
    pub artists: Vec<String>,
    pub mappers: Vec<String>,
    pub audio: Vec<u8>,
    pub cover: Vec<u8>,
    pub notes: Vec<Note>,
    pub objects: Vec<ObjectDefinition>,
}

pub struct PartialMap {
    pub title: String,
    pub mappers: Vec<String>,
    pub artists: Vec<String>,
}

impl MapMeta for Map {
    fn get_title(&self) -> String {
        self.title.clone()
    }

    fn get_mappers(&self) -> Vec<String> {
        self.mappers.clone()
    }

    fn get_artists(&self) -> Vec<String> {
        self.artists.clone()
    }

    fn get_length(&self) -> u32 {
        self.length
    }
}
