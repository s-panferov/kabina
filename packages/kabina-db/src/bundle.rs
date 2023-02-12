use std::path::PathBuf;

use crate::DependencyKind;

#[derive(Debug, Clone)]
pub struct BundleItem {
    prefix: PathBuf,
    content: DependencyKind,
}

#[salsa::input]
#[derive(Debug, Clone)]
pub struct Bundle {
    name: String,
    items: Vec<BundleItem>,
}
