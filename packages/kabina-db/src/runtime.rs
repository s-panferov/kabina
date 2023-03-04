use std::path::PathBuf;

use crate::{File, Schema, TransformApply};

pub trait Runtime {
    async fn load_schema(&mut self, schema: PathBuf) -> Schema;
    async fn transform(&mut self, task: &TransformApply) -> File;
}
