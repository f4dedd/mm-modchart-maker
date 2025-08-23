pub mod note;

pub trait MapObject {
    fn get_millisecond(&self) -> u32;
}
