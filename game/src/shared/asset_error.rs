
use thiserror::Error;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum BlobAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load file: {0}")]
    Io(#[from] std::io::Error),
}