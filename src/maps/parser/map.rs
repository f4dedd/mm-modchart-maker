use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read, Seek},
    path::PathBuf,
};

use bevy::math::{Vec2, Vec3};

use crate::maps::parser::io::BinaryReader;
use crate::maps::{map::Map, objects::note::Note};

pub trait MapParser {
    fn parse(path: PathBuf) -> io::Result<Map>;
}

pub trait ObjectParser {
    fn from_definition(definition: ObjectDefinition) -> io::Result<Self>
    where
        Self: Sized;
}

#[derive(Debug)]
pub struct ObjectDefinition {
    pub name: String,
    pub millisecond: u32,
    pub definitions: Vec<ObjectType>,
}

#[derive(Debug)]
pub enum ObjectType {
    U8(Option<u8>),
    U16(Option<u16>),
    U32(Option<u32>),
    U64(Option<u64>),
    F32(Option<f32>),
    F64(Option<f64>),
    Vec2(Option<Vec2>),
    Buf(Option<Vec<u8>>),
    String(Option<String>),
    LongBuf(Option<Vec<u8>>),
    LongString(Option<String>),
    I64(Option<i64>),
    Vec3(Option<Vec3>),
}

pub struct SSPMParser;

impl MapParser for SSPMParser {
    fn parse(path: PathBuf) -> io::Result<Map> {
        let mut parser = BinaryReader::new(File::open(path)?);

        // Header structure:
        // The first 4 bytes are the file signature "SS+m"
        // The next 2 bytes are the version of the sspm (currently only version 2 is supported)
        // The rest of the header is unused
        let mut header = [0u8; 10];
        parser.read_exact(&mut header)?;

        // Header signature must be "SS+m"
        if header[0..4] != [0x53, 0x53, 0x2B, 0x6D] {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Incorrect file signature",
            ));
        }

        // Version of sspm must be 2
        if header[4..6] != [0x02, 0x00] {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unsupported SSPM version",
            ));
        }

        let _hash = parser.read_sha1()?;
        let _millisecond = parser.read_u32()?;
        let _note_count = parser.read_u32()?;
        let _marker_count = parser.read_u32()?;
        let _difficulty = parser.read_u8()?;
        let _star_rating = parser.read_u16()?;

        let has_audio = parser.read_bool()?;
        let has_cover = parser.read_bool()?;
        let _has_mod = parser.read_bool()?;

        let _custom_data_length = parser.read_u64()?;
        let _custom_data_offset = parser.read_u64()?;
        let audio_data_length = parser.read_u64()?;
        let audio_data_offset = parser.read_u64()?;
        let cover_data_length = parser.read_u64()?;
        let cover_data_offset = parser.read_u64()?;
        let object_definition_offset = parser.read_u64()?;
        let _object_definition_length = parser.read_u64()?;
        let object_data_offset = parser.read_u64()?;
        let object_data_length = parser.read_u64()?;

        let _map_id = parser.read_string()?;
        let _map_name = parser.read_string()?;
        let song_name = parser.read_string()?;
        let mappers_count = parser.read_u16()?;
        let mut mappers = Vec::<String>::new();

        for _ in 0..mappers_count {
            mappers.push(parser.read_string()?);
        }

        let mut audio_buf = vec![0u8; audio_data_length as usize];
        let mut cover_buf = vec![0u8; cover_data_length as usize];

        if has_audio {
            parser.seek(io::SeekFrom::Start(audio_data_offset))?;
            parser.read_exact(&mut audio_buf)?;
        }

        if has_cover {
            parser.seek(io::SeekFrom::Start(cover_data_offset))?;
            parser.read_exact(&mut cover_buf)?;
        }

        let mut object_definitions = HashMap::<u8, ObjectDefinition>::new();

        parser.seek(io::SeekFrom::Start(object_definition_offset))?;

        let object_count = parser.read_u8()?;

        for count in 0..object_count {
            let name = parser.read_string()?;
            let values = parser.read_u8()?;

            let mut definitions = Vec::<ObjectType>::new();

            for _ in 0..values {
                let obj_type = parser.read_u8()?;
                let data = ObjectType::from_sspm(obj_type)?;

                definitions.push(data);
            }

            // There should be an empty byte after each object definition
            assert_eq!(parser.read_u8()?, 0x00);

            object_definitions.insert(
                count,
                ObjectDefinition {
                    name,
                    millisecond: 0,
                    definitions,
                },
            );
        }

        parser.seek(io::SeekFrom::Start(object_data_offset))?;
        let object_section_end = parser.stream_position()? + object_data_length;

        let mut notes = Vec::<Note>::new();
        let mut undefined_objects = Vec::<ObjectDefinition>::new();

        while parser.stream_position()? < object_section_end {
            let ms = parser.read_u32()?;
            let definition = parser.read_u8()?;

            let object =
                SSPMParser::parse_definitions(&object_definitions[&definition], ms, &mut parser)?;

            match object.name.as_str() {
                "ssp_note" => notes.push(Note::from_definition(object)?),
                _ => undefined_objects.push(object),
            }
        }

        Ok(Map {
            title: song_name,
            artists: vec![],
            mappers,
            audio: audio_buf,
            cover: cover_buf,
            notes: notes,
            objects: undefined_objects,
        })
    }
}

impl SSPMParser {
    fn parse_definitions<T: Read + Seek>(
        marker_definition: &ObjectDefinition,
        ms: u32,
        parser: &mut BinaryReader<T>,
    ) -> io::Result<ObjectDefinition> {
        let mut object_types = Vec::<ObjectType>::new();

        for def in marker_definition.definitions.iter() {
            match def {
                ObjectType::U8(_) => object_types.push(Self::parse_u8(parser)?),
                ObjectType::U16(_) => object_types.push(Self::parse_u16(parser)?),
                ObjectType::U32(_) => object_types.push(Self::parse_u32(parser)?),
                ObjectType::U64(_) => object_types.push(Self::parse_u64(parser)?),
                ObjectType::F32(_) => object_types.push(Self::parse_f32(parser)?),
                ObjectType::F64(_) => object_types.push(Self::parse_f64(parser)?),
                ObjectType::Vec2(_) => object_types.push(Self::parse_vec2(parser)?),
                ObjectType::Buf(_) => object_types.push(Self::parse_buf(parser)?),
                ObjectType::LongBuf(_) => object_types.push(Self::parse_long_buf(parser)?),
                ObjectType::String(_) => object_types.push(Self::parse_string(parser)?),
                ObjectType::LongString(_) => object_types.push(Self::parse_long_string(parser)?),
                _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "")),
            }
        }

        Ok(ObjectDefinition {
            name: marker_definition.name.clone(),
            millisecond: ms,
            definitions: object_types,
        })
    }

    fn parse_u8<T: Read + Seek>(parser: &mut BinaryReader<T>) -> io::Result<ObjectType> {
        Ok(ObjectType::U8(Some(parser.read_u8()?)))
    }

    fn parse_u16<T: Read + Seek>(parser: &mut BinaryReader<T>) -> io::Result<ObjectType> {
        Ok(ObjectType::U16(Some(parser.read_u16()?)))
    }

    fn parse_u32<T: Read + Seek>(parser: &mut BinaryReader<T>) -> io::Result<ObjectType> {
        Ok(ObjectType::U32(Some(parser.read_u32()?)))
    }

    fn parse_u64<T: Read + Seek>(parser: &mut BinaryReader<T>) -> io::Result<ObjectType> {
        Ok(ObjectType::U64(Some(parser.read_u64()?)))
    }

    fn parse_f32<T: Read + Seek>(parser: &mut BinaryReader<T>) -> io::Result<ObjectType> {
        Ok(ObjectType::F32(Some(parser.read_f32()?)))
    }

    fn parse_f64<T: Read + Seek>(parser: &mut BinaryReader<T>) -> io::Result<ObjectType> {
        Ok(ObjectType::F64(Some(parser.read_f64()?)))
    }

    fn parse_vec2<T: Read + Seek>(parser: &mut BinaryReader<T>) -> io::Result<ObjectType> {
        let quantum = parser.read_bool()?;
        let mut pos = Vec2::ZERO;

        match quantum {
            true => {
                pos.x = parser.read_f32()?;
                pos.y = parser.read_f32()?;
            }
            false => {
                pos.x = parser.read_u8()? as f32;
                pos.y = parser.read_u8()? as f32;
            }
        };

        Ok(ObjectType::Vec2(Some(pos)))
    }

    fn parse_buf<T: Read + Seek>(parser: &mut BinaryReader<T>) -> io::Result<ObjectType> {
        let mut length = [0u8; 2];
        parser.read_exact(&mut length)?;
        let mut buffer = vec![0u8; u16::from_le_bytes(length) as usize];
        parser.read_exact(&mut buffer)?;

        Ok(ObjectType::Buf(Some(buffer)))
    }

    fn parse_long_buf<T: Read + Seek>(parser: &mut BinaryReader<T>) -> io::Result<ObjectType> {
        let mut length = [0u8; 4];
        parser.read_exact(&mut length)?;
        let mut buffer = vec![0u8; u32::from_le_bytes(length) as usize];
        parser.read_exact(&mut buffer)?;

        Ok(ObjectType::LongBuf(Some(buffer)))
    }

    fn parse_string<T: Read + Seek>(parser: &mut BinaryReader<T>) -> io::Result<ObjectType> {
        Ok(ObjectType::String(Some(parser.read_string()?)))
    }

    fn parse_long_string<T: Read + Seek>(parser: &mut BinaryReader<T>) -> io::Result<ObjectType> {
        Ok(ObjectType::LongString(Some(parser.read_long_string()?)))
    }
}

impl ObjectType {
    pub fn from_sspm(value: u8) -> io::Result<ObjectType> {
        match value {
            0x01 => Ok(ObjectType::U8(None)),
            0x02 => Ok(ObjectType::U16(None)),
            0x03 => Ok(ObjectType::U32(None)),
            0x04 => Ok(ObjectType::U64(None)),
            0x05 => Ok(ObjectType::F32(None)),
            0x06 => Ok(ObjectType::F64(None)),
            0x07 => Ok(ObjectType::Vec2(None)),
            0x08 => Ok(ObjectType::Buf(None)),
            0x09 => Ok(ObjectType::String(None)),
            0x0A => Ok(ObjectType::LongBuf(None)),
            0x0B => Ok(ObjectType::LongString(None)),
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, "")),
        }
    }
}
