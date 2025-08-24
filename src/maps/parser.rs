use std::{
    collections::HashMap,
    io::{self, Cursor, Read, Seek, SeekFrom, Write},
};

use bevy::{
    audio::AudioSource,
    math::{Vec2, Vec3, ops::round},
};
use serde::{Deserialize, Serialize};

use crate::maps::{Map, objects::Note};
use crate::maps::{
    MapFormat,
    io::{BinaryReader, BinaryWriter},
};

pub struct SSPMSerializer;

pub struct PHXMParser;

pub trait MapSerializer {
    fn deserialize<T: Read + Seek>(reader: T) -> io::Result<Map>;
    fn serialize<T: Write + Seek>(map: &Map, writer: T) -> io::Result<()>;
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
    Vec(Option<Vec<ObjectType>>),
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
            0x0C => Ok(ObjectType::Vec(None)),
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, "")),
        }
    }
}

impl MapSerializer for SSPMSerializer {
    fn serialize<T: Write + Seek>(map: &Map, writer: T) -> io::Result<()> {
        let mut writer = BinaryWriter::new(writer);

        // Header
        writer.write_all(b"SS+m")?; // File signature
        writer.write_all(&[0x02, 0x00])?; // Version 2
        writer.write_all(&[0u8; 4])?; // Unused bytes

        // Static Metadata
        writer.write_sha1(&[0u8; 20])?; // SHA1 is never used yet so ignore for now
        writer.write_u32(map.length)?;
        writer.write_u32(map.notes.len() as u32)?;
        writer.write_u32((map.notes.len() + map.objects.len()) as u32)?;

        writer.write_u8(map.difficulty)?;
        writer.write_u16(0)?; // Star rating is never used
        writer.write_bool(map.audio.is_some())?;
        writer.write_bool(!map.cover.is_empty())?;
        writer.write_bool(false)?;

        let data_offset = writer.stream_position()?;
        writer.write_all(&[0u8; 80])?; // Placeholder for data offsets and lengths

        writer.write_string(&map.id)?;
        writer.write_string(&map.title)?;
        writer.write_string(&map.title)?; // Song name is the same as title for now

        writer.write_u16(map.mappers.len() as u16)?;
        for mapper in map.mappers.iter() {
            writer.write_string(mapper)?;
        }

        let mut custom_data_offset: u64 = 0;
        let mut custom_data_length: u64 = 0;

        if !map.difficulty_name.is_empty() {
            custom_data_offset = writer.stream_position()?;

            writer.write_u16(1)?; // One custom data field
            writer.write_string("difficulty_name")?;
            writer.write_u8(0x09)?; // String type
            writer.write_string(&map.difficulty_name)?;

            custom_data_length = writer.stream_position()? - custom_data_offset;
        } else {
            writer.write_u16(0)?; // zero custom data fields
        }

        let audio_offset = writer.stream_position()?;
        if let Some(audio) = &map.audio {
            writer.write_all(&audio.bytes)?;
        }
        let audio_length = writer.stream_position()? - audio_offset;

        let mut cover_offset = 0;
        let mut cover_length = 0;

        if !map.cover.is_empty() {
            cover_offset = writer.stream_position()?;
            writer.write_all(&map.cover)?;
            cover_length = writer.stream_position()? - cover_offset;
        }

        let object_definition_offset = writer.stream_position()?;
        writer.write_u8(1)?;
        writer.write_string("ssp_note")?;
        writer.write_all(&[0x01, 0x07, 0x00])?; // One definition of type Vec2
        let object_definition_length = writer.stream_position()? - object_definition_offset;

        let object_data_offset = writer.stream_position()?;

        for note in map.notes.iter() {
            writer.write_u32(note.millisecond)?;
            writer.write_u8(0x00)?;

            let quantum = round(note.position.x) != round_to_places(note.position.x, 2)
                || round(note.position.y) != round_to_places(note.position.y, 2);

            writer.write_bool(quantum)?;

            if quantum {
                writer.write_f32(note.position.x)?;
                writer.write_f32(note.position.y)?;
            } else {
                writer.write_u8(note.position.x as u8 + 1)?;
                writer.write_u8(note.position.y as u8 + 1)?;
            }
        }

        let object_data_length = writer.stream_position()? - object_data_offset;

        writer.seek(SeekFrom::Start(data_offset))?;
        writer.write_u64(custom_data_offset)?;
        writer.write_u64(custom_data_length)?;
        writer.write_u64(audio_offset)?;
        writer.write_u64(audio_length)?;
        writer.write_u64(cover_offset)?;
        writer.write_u64(cover_length)?;
        writer.write_u64(object_definition_offset)?;
        writer.write_u64(object_definition_length)?;
        writer.write_u64(object_data_offset)?;
        writer.write_u64(object_data_length)?;

        writer.seek(SeekFrom::End(0))?;
        writer.write_string(format!("MM Export - {}", "0.0.1").as_str())?;

        Ok(())
    }

    fn deserialize<T: Read + Seek>(reader: T) -> io::Result<Map> {
        let mut reader = BinaryReader::new(reader);

        // Header structure:
        // The first 4 bytes are the file signature "SS+m"
        // The next 2 bytes are the version of the sspm (currently only version 2 is supported)
        // The rest of the header is unused
        let mut header = [0u8; 10];
        reader.read_exact(&mut header)?;

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

        let _hash = reader.read_sha1()?; // SHA1 hash of the file
        let millisecond = reader.read_u32()?; // Last object millisecond
        let _note_count = reader.read_u32()?; // Note object count
        let _object_count = reader.read_u32()?; // Total object count ( including notes )
        //
        let _difficulty = reader.read_u8()?;
        let _star_rating = reader.read_u16()?; // never used
        let has_audio = reader.read_bool()?; // Whether the map has audio data
        let has_cover = reader.read_bool()?; // Whether the map has cover data
        let _has_mod = reader.read_bool()?; // never used

        let _custom_data_offset = reader.read_u64()?; // never used
        let _custom_data_length = reader.read_u64()?; // never used
        let audio_data_offset = reader.read_u64()?; // Length of audio data
        let audio_data_length = reader.read_u64()?; // Offset of audio data
        let cover_data_offset = reader.read_u64()?; // Length of cover data
        let cover_data_length = reader.read_u64()?; // Offset of cover data
        let object_definition_offset = reader.read_u64()?; // Offset of object definitions
        let _object_definition_length = reader.read_u64()?; // Length of object definitions
        let object_data_offset = reader.read_u64()?; // Offset of object data
        let object_data_length = reader.read_u64()?; // Length of object data

        let map_id = reader.read_string()?; // Id of the map
        let _map_name = reader.read_string()?; // Name of the map
        let song_name = reader.read_string()?; // Song name
        let mappers_count = reader.read_u16()?; // Mappers count
        let mut mappers = Vec::<String>::new();

        for _ in 0..mappers_count {
            mappers.push(reader.read_string()?);
        }

        let custom_data_fields = reader.read_u16()?;

        let mut custom_data = HashMap::<String, ObjectType>::new();

        for _ in 0..custom_data_fields {
            let name = reader.read_string()?;
            let data_type = ObjectType::from_sspm(reader.read_u8()?)?;
            let value = SSPMSerializer::parse_types(&data_type, &mut reader)?;

            custom_data.insert(name, value);
        }

        let mut audio_buf = vec![0u8; audio_data_length as usize];
        let mut cover_buf = vec![0u8; cover_data_length as usize];

        if has_audio {
            reader.seek(io::SeekFrom::Start(audio_data_offset))?;
            reader.read_exact(&mut audio_buf)?;
        }

        if has_cover {
            reader.seek(io::SeekFrom::Start(cover_data_offset))?;
            reader.read_exact(&mut cover_buf)?;
        }

        let mut object_definitions = HashMap::<u8, ObjectDefinition>::new();

        reader.seek(io::SeekFrom::Start(object_definition_offset))?;

        let object_count = reader.read_u8()?;

        for count in 0..object_count {
            let name = reader.read_string()?;
            let values = reader.read_u8()?;

            let mut definitions = Vec::<ObjectType>::new();

            for _ in 0..values {
                let obj_type = reader.read_u8()?;
                let data = ObjectType::from_sspm(obj_type)?;

                definitions.push(data);
            }

            // There should be an empty byte after each object definition
            assert_eq!(reader.read_u8()?, 0x00);

            object_definitions.insert(
                count,
                ObjectDefinition {
                    name,
                    millisecond: 0,
                    definitions,
                },
            );
        }

        reader.seek(io::SeekFrom::Start(object_data_offset))?;
        let object_section_end = reader.stream_position()? + object_data_length;

        let mut notes = Vec::<Note>::new();
        let mut objects = Vec::<ObjectDefinition>::new();

        while reader.stream_position()? < object_section_end {
            let ms = reader.read_u32()?;
            let definition = reader.read_u8()?;

            let object = SSPMSerializer::parse_definitions(
                &object_definitions[&definition],
                ms,
                &mut reader,
            )?;

            match object.name.as_str() {
                "ssp_note" => notes.push(Note::from_definition(object)?),
                _ => objects.push(object),
            }
        }

        let audio_source = match audio_buf.is_empty() {
            true => None,
            false => Some(AudioSource {
                bytes: audio_buf.into(),
            }),
        };

        Ok(Map {
            id: map_id,
            length: millisecond,
            title: song_name,
            artists: vec![],
            difficulty: 0,
            difficulty_name: String::new(),
            mappers,
            audio: audio_source,
            cover: cover_buf,
            notes,
            objects,
            format: MapFormat::SSPM,
        })
    }
}
fn round_to_places(value: f32, places: u32) -> f32 {
    let factor = 10f32.powi(places as i32);
    (value * factor).round() / factor
}

impl SSPMSerializer {
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

    fn parse_types<T: Read + Seek>(
        object_type: &ObjectType,
        parser: &mut BinaryReader<T>,
    ) -> io::Result<ObjectType> {
        match object_type {
            ObjectType::U8(_) => Self::parse_u8(parser),
            ObjectType::U16(_) => Self::parse_u16(parser),
            ObjectType::U32(_) => Self::parse_u32(parser),
            ObjectType::U64(_) => Self::parse_u64(parser),
            ObjectType::F32(_) => Self::parse_f32(parser),
            ObjectType::F64(_) => Self::parse_f64(parser),
            ObjectType::Vec2(_) => Self::parse_vec2(parser),
            ObjectType::Buf(_) => Self::parse_buf(parser),
            ObjectType::LongBuf(_) => Self::parse_long_buf(parser),
            ObjectType::String(_) => Self::parse_string(parser),
            ObjectType::LongString(_) => Self::parse_long_string(parser),
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, "")),
        }
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
                pos.x = (parser.read_u8()? as f32) - 2.0;
                pos.y = (parser.read_u8()? as f32) - 2.0;
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

    fn parse_vec<T: Read + Seek>(_parser: &mut BinaryReader<T>) -> io::Result<ObjectType> {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Not implemented",
        ))
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PHXMMetadata {
    #[serde(rename = "ID")]
    id: String,
    has_audio: bool,
    has_cover: bool,
    has_video: bool,
    audio_extension: String,
    artist: String,
    title: String,
    mappers: Vec<String>,
    difficulty: u8,
    difficulty_name: String,
    notes_count: u32,
}

impl MapSerializer for PHXMParser {
    fn serialize<T: Write + Seek>(_map: &Map, _writer: T) -> io::Result<()> {
        todo!()
    }

    fn deserialize<T: Read + Seek>(reader: T) -> io::Result<Map> {
        let mut folder = zip::ZipArchive::new(reader)?;
        let mut parser: BinaryReader<Cursor<Vec<u8>>>;

        let mut audio_buf = Vec::<u8>::new();
        let mut cover_buf = Vec::<u8>::new();
        let mut video_buf = Vec::<u8>::new();
        let metadata: PHXMMetadata;
        let mut notes: Vec<Note>;

        {
            let mut file = folder.by_name("metadata.json")?;
            let mut buf = String::new();
            file.read_to_string(&mut buf)?;

            metadata = serde_json::from_str(&buf)?;
        }

        {
            let mut file = folder.by_name("objects.phxmo")?;
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;

            let mut cursor = Cursor::new(buf);
            cursor.seek(SeekFrom::Start(0))?;
            parser = BinaryReader::new(cursor);
        }

        if metadata.has_audio {
            folder
                .by_name(format!("audio.{}", metadata.audio_extension).as_str())?
                .read_to_end(&mut audio_buf)?;
        }

        if metadata.has_cover {
            folder.by_name("cover.png")?.read_to_end(&mut cover_buf)?;
        }

        if metadata.has_video {
            folder.by_name("video.mp4")?.read_to_end(&mut video_buf)?;
        }

        let _type_count = parser.read_u32()?;
        let note_count = parser.read_u32()?;

        notes = Vec::new();

        for _ in 0..note_count {
            let millisecond = parser.read_u32()?;
            let quantum = parser.read_bool()?;

            match quantum {
                true => {
                    let x = parser.read_f32()?;
                    let y = parser.read_f32()?;

                    notes.push(Note {
                        millisecond,
                        position: Vec2::new(x, y),
                    });
                }
                false => {
                    let x = (parser.read_u8()? - 1) as f32;
                    let y = (parser.read_u8()? - 1) as f32;

                    notes.push(Note {
                        millisecond,
                        position: Vec2::new(x, y),
                    });
                }
            }
        }

        let audio_source = match audio_buf.is_empty() {
            true => None,
            false => Some(AudioSource {
                bytes: audio_buf.clone().into(),
            }),
        };

        Ok(Map {
            id: metadata.id,
            length: notes.last().map_or(0, |n| n.millisecond),
            title: metadata.title,
            artists: vec![metadata.artist],
            difficulty: metadata.difficulty,
            difficulty_name: metadata.difficulty_name,
            mappers: metadata.mappers,
            audio: audio_source,
            cover: cover_buf,
            notes: notes,
            objects: vec![],
            format: MapFormat::PHXM,
        })
    }
}
