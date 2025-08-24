pub mod io;
pub mod map;
pub mod objects;
pub mod parser;

use bevy::{
    asset::{io::Reader, *},
    prelude::*,
};
use std::io::Cursor;

pub use map::*;

use crate::maps::parser::{MapSerializer, SSPMSerializer};

#[derive(Resource)]
pub struct MapFolder(pub Handle<LoadedFolder>);

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Map>().init_asset_loader::<SSPMLoader>();
    }
}

#[derive(Default)]
pub struct SSPMLoader;

impl AssetLoader for SSPMLoader {
    type Asset = Map;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await?;
        let cursor = Cursor::new(buf);
        let map = SSPMSerializer::deserialize(cursor)?;

        Ok(map)
    }

    fn extensions(&self) -> &[&str] {
        &["sspm"]
    }
}
