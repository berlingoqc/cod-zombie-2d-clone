use bevy::{
    asset::{AssetLoader, LoadContext },
};

use crate::shared::asset_error::BlobAssetLoaderError;

use super::render::MapDataAsset;


#[derive(Default)]
pub struct MapDataAssetLoader;

impl AssetLoader for MapDataAssetLoader {

    type Asset = MapDataAsset;
    type Settings = ();
    type Error = BlobAssetLoaderError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a (),
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        
            println!("Importing data");
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            let map_data_asset = ron::de::from_bytes::<MapDataAsset>(&bytes)?;

            Ok(map_data_asset)
        
    }

    fn extensions(&self) -> &[&str] {
        &["asset.ron"]
    }
}
