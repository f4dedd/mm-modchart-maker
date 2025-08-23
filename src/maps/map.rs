use std::io;

use bevy::math::Vec2;

use crate::maps::objects::note::Note;

use super::parser::map::{ObjectDefinition, ObjectParser, ObjectType};

pub trait MapInfo {
    fn get_title(&self) -> String;
    fn get_mappers(&self) -> Vec<String>;
    fn get_artists(&self) -> Vec<String>;
}

#[derive(Debug)]
pub struct Map {
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
