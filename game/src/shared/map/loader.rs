use bevy::{
    asset::{AssetLoader, LoadContext },
};

use thiserror::Error;

use super::render::MapDataAsset;

//#[non_exhaustive]
//#[derive(Debug, Error)]
//enum CustomAssetLoaderError {
    /// An [IO](std::io) Error
    //#[error("Could not load asset: {0}")]
    //Io(#[from] std::io::Error),
    /// A [RON](ron) Error
    //#[error("Could not parse RON: {0}")]
    //RonSpannedError(#[from] ron::error::SpannedError),
//}

#[non_exhaustive]
#[derive(Debug, Error)]
enum BlobAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load file: {0}")]
    Io(#[from] std::io::Error),
}


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
