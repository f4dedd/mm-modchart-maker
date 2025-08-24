pub mod note;

pub use note::*;

pub trait MapObject {
    fn get_millisecond(&self) -> u32;
}
