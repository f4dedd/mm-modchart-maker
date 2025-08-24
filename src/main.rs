use bevy::prelude::*;
use std;

mod io;
mod jukebox;
mod maps;
mod player;

const _UPDATE_FREQUENCY: f32 = 1.0 / 60.0; // 60 updates per second

fn main() -> std::io::Result<()> {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugins(maps::MapPlugin)
        .add_plugins(player::PlayerPlugin)
        .add_systems(Startup, load_assets)
        .add_systems(FixedPostUpdate, get_maps)
        .run();

    Ok(())
}

pub fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(maps::MapFolder(asset_server.load_folder("maps")));
}

pub fn get_maps(maps: Res<Assets<maps::Map>>) {
    println!("Maps loaded: {}", maps.len());
}
