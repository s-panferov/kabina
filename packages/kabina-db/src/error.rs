use std::sync::Arc;

use by_address::ByAddress;
use thiserror::Error;

#[derive(Error, Clone, Eq, PartialEq, Debug)]
pub enum Cause {
    #[error("Data is not available")]
    Pending,
    #[error("Generic {0}")]
    Error(#[from] Arc<ByAddress<anyhow::Error>>),
}

impl Cause {
    pub fn from_err(e: impl std::error::Error + Send + Sync + 'static) -> Cause {
        Cause::Error(Arc::new(ByAddress(e.into())))
    }
}

pub type Outcome<T> = std::result::Result<T, Cause>;
