use std::io;

use bevy::math::Vec2;

use crate::maps::{
    objects::MapObject,
    parser::{ObjectDefinition, ObjectParser, ObjectType},
};

#[derive(Debug)]
pub struct Note {
    pub millisecond: u32,
    pub position: Vec2,
}

impl MapObject for Note {
    fn get_millisecond(&self) -> u32 {
        self.millisecond
    }
}

impl ObjectParser for Note {
    fn from_definition(obj: ObjectDefinition) -> io::Result<Self> {
        if obj.definitions.len() < 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Object definition has no types",
            ));
        }

        let pos = match obj.definitions[0] {
            ObjectType::Vec2(Some(v)) => v,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Object could not be parsed as Note",
                ));
            }
        };

        Ok(Note {
            millisecond: obj.millisecond,
            position: pos,
        })
    }
}
