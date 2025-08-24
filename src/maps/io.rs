use std::io::{self, Read, Seek, SeekFrom, Write};

use bevy::{app::PanicHandlerPlugin, math::Vec2};

pub struct BinaryReader<T: Read + Seek> {
    reader: T,
}

pub struct BinaryWriter<T: Write> {
    writer: T,
}

impl<T: Seek + Read> BinaryReader<T> {
    pub fn new(reader: T) -> Self {
        Self { reader }
    }

    pub fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.reader.seek(pos)
    }

    pub fn stream_position(&mut self) -> io::Result<u64> {
        self.reader.stream_position()
    }

    pub fn read_bool(&mut self) -> io::Result<bool> {
        let mut buf = [0u8; 1];
        self.reader.read_exact(&mut buf)?;
        Ok(buf[0] == 0x01)
    }

    pub fn read_u8(&mut self) -> io::Result<u8> {
        let mut buf = [0u8; 1];
        self.reader.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    pub fn read_u16(&mut self) -> io::Result<u16> {
        let mut buf = [0u8; 2];
        self.reader.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    pub fn read_u32(&mut self) -> io::Result<u32> {
        let mut buf = [0u8; 4];
        self.reader.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    pub fn read_u64(&mut self) -> io::Result<u64> {
        let mut buf = [0u8; 8];
        self.reader.read_exact(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }

    pub fn read_f32(&mut self) -> io::Result<f32> {
        let mut buf = [0u8; 4];
        self.reader.read_exact(&mut buf)?;
        Ok(f32::from_le_bytes(buf))
    }

    pub fn read_f64(&mut self) -> io::Result<f64> {
        let mut buf = [0u8; 8];
        self.reader.read_exact(&mut buf)?;
        Ok(f64::from_le_bytes(buf))
    }

    pub fn read_string(&mut self) -> io::Result<String> {
        let mut buf = [0u8; 2];
        self.reader.read_exact(&mut buf)?;
        let mut buffer = vec![0u8; u16::from_le_bytes(buf) as usize];
        self.reader.read_exact(&mut buffer)?;

        let str = String::from_utf8(buffer);

        match str {
            Ok(s) => Ok(s),
            Err(_) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid UTF-8 sequence",
            )),
        }
    }

    pub fn read_long_string(&mut self) -> io::Result<String> {
        let mut buf = [0u8; 4];
        self.reader.read_exact(&mut buf)?;
        let mut buffer = vec![0u8; u32::from_le_bytes(buf) as usize];
        self.reader.read_exact(&mut buffer)?;

        let str = String::from_utf8(buffer);

        match str {
            Ok(s) => Ok(s),
            Err(_) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid UTF-8 sequence",
            )),
        }
    }

    pub fn read_sha1(&mut self) -> io::Result<[u8; 20]> {
        let mut buf = [0u8; 20];
        self.reader.read_exact(&mut buf)?;
        Ok(buf)
    }

    pub fn read_vec2(&mut self) -> io::Result<bevy::math::Vec2> {
        let quantum = self.read_bool()?;
        let mut pos = Vec2::ZERO;

        match quantum {
            true => {
                pos.x = self.read_f32()?;
                pos.y = self.read_f32()?;
            }
            false => {
                pos.x = self.read_u8()? as f32;
                pos.y = self.read_u8()? as f32;
            }
        }

        Ok(pos)
    }

    pub fn read_vec3(&mut self) -> io::Result<bevy::math::Vec3> {
        let quantum = self.read_bool()?;
        let mut pos = bevy::math::Vec3::ZERO;

        match quantum {
            true => {
                pos.x = self.read_f32()?;
                pos.y = self.read_f32()?;
                pos.z = self.read_f32()?;
            }
            false => {
                pos.x = self.read_u8()? as f32;
                pos.y = self.read_u8()? as f32;
                pos.z = self.read_u8()? as f32;
            }
        }

        Ok(pos)
    }

    pub fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.reader.read_exact(buf)
    }
}

impl<T: Write> BinaryWriter<T> {
    pub fn new(writer: T) -> Self {
        Self { writer }
    }

    pub fn write_bool(&mut self, value: bool) -> io::Result<()> {
        let byte = if value { 0x01 } else { 0x00 };
        self.writer.write_all(&[byte])
    }

    pub fn write_u8(&mut self, value: u8) -> io::Result<()> {
        self.writer.write_all(&[value])
    }

    pub fn write_u16(&mut self, value: u16) -> io::Result<()> {
        self.writer.write_all(&value.to_le_bytes())
    }

    pub fn write_u32(&mut self, value: u32) -> io::Result<()> {
        self.writer.write_all(&value.to_le_bytes())
    }

    pub fn write_u64(&mut self, value: u64) -> io::Result<()> {
        self.writer.write_all(&value.to_le_bytes())
    }

    pub fn write_f32(&mut self, value: f32) -> io::Result<()> {
        self.writer.write_all(&value.to_le_bytes())
    }

    pub fn write_f64(&mut self, value: f64) -> io::Result<()> {
        self.writer.write_all(&value.to_le_bytes())
    }

    pub fn write_string(&mut self, value: &str) -> io::Result<()> {
        let len = value.len() as u16;
        self.writer.write_all(&len.to_le_bytes())?;
        self.writer.write_all(value.as_bytes())
    }

    pub fn write_long_string(&mut self, value: &str) -> io::Result<()> {
        let len = value.len() as u32;
        self.writer.write_all(&len.to_le_bytes())?;
        self.writer.write_all(value.as_bytes())
    }

    pub fn write_sha1(&mut self, value: &[u8; 20]) -> io::Result<()> {
        self.writer.write_all(value)
    }

    pub fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.writer.write_all(buf)
    }
}
