use std::{path::PathBuf, sync::Arc};

use deno_core::{op, OpState};
use kabina_db::{Bundle, Database, SchemaBuilder};
use serde::Deserialize;

use crate::transform::JsDependency;

#[derive(Deserialize)]
pub struct JsBundleItem {
    prefix: PathBuf,
    content: JsDependency,
}

#[derive(Deserialize)]
pub struct JsBundle {
    name: String,
    items: Vec<JsBundleItem>,
}

#[op]
pub fn bundle(state: &mut OpState, b: JsBundle) -> Result<f64, deno_core::error::AnyError> {
    tracing::info!("Bundle {:?} created ", b.name);

    let db = state.borrow::<Arc<Database>>();
    let schema = state.borrow::<Arc<SchemaBuilder>>();

    let handle = Bundle::new(&**db, b.name, vec![]);

    schema.register_bundle(handle);

    Ok(usize::from(kabina_db::AsId::as_id(handle)) as f64)
}
