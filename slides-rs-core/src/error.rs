use thiserror::Error;

#[derive(Debug, Error)]
pub enum SlidesError {
    #[error("{0}")]
    IoError(#[from] std::io::Error),
}
