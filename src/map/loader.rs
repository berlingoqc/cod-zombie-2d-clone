use bevy::{prelude::*, asset::{AssetLoader, LoadContext, BoxedFuture, LoadedAsset}};

use super::data::MapDataAsset;

#[derive(Default)]
pub struct MapDataAssetLoader;

impl AssetLoader for MapDataAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            println!("Importing data");
            let map_data_asset = ron::de::from_bytes::<MapDataAsset>(bytes)?;

            load_context.set_default_asset(LoadedAsset::new(map_data_asset));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["custom"]
    }
}


