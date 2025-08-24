use bevy::prelude::*;

use crate::player::mods::Mods;

#[derive(Bundle)]
pub struct Game {
    mods: Mods,
    song: AudioPlayer,
}
