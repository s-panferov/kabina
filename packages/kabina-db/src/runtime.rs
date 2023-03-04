use std::path::PathBuf;

use crate::{ApplyTransform, File, Schema};

pub trait Runtime {
    async fn load_schema(&mut self, schema: PathBuf) -> Schema;
    async fn transform(&mut self, task: &ApplyTransform) -> File;
}
