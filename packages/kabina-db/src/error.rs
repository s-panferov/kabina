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

pub type Outcome<T> = std::result::Result<T, Cause>;
