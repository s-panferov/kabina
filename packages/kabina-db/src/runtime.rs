use url::Url;

use crate::{File, Schema, TransformApply};

pub trait Runtime {
	async fn load_schema(&mut self, schema: Url) -> Schema;
	async fn transform(&mut self, task: &TransformApply) -> File;
}
