use bevy::prelude::*;


use super::{data::{MapDataAsset, WallBundle}, loader::MapDataAssetLoader};


impl MapDataAsset {
    pub fn render(&self, command: &mut Commands) {
        for w in (&self.walls).into_iter() {
            command.spawn_bundle(WallBundle::new(w.clone()));
        }
    }
}

