use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Codec(#[from] bincode::Error),

    #[error("reassembly failed: {0:?}")]
    Reassemble(spex_core::reassemble::ReassembleError),
}

impl From<spex_core::reassemble::ReassembleError> for NetError {
    fn from(value: spex_core::reassemble::ReassembleError) -> Self {
        NetError::Reassemble(value)
    }
}

pub type Result<T> = std::result::Result<T, NetError>;
