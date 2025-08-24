use bevy::prelude::*;

mod game;
mod mods;

use game::*;
use mods::*;

#[derive(States, PartialEq, Eq, Debug, Hash, Clone, Default)]
pub enum SimulationState {
    #[default]
    Paused,
    Running,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SimulationState>();
    }
}
