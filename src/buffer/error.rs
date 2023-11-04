use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error("no free buffer is available in this buffer pool.")]
    NoFreeBuffer,
}
