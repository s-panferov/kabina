use std::{path::PathBuf, sync::Arc};

use deno_core::{op, OpState};
use kabina_db::{Bundle, BundleItem, SchemaBuilder, SharedDatabase};
use serde::Deserialize;

use crate::transform::{map_js_dep, JsDependency};

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

    let db = state.borrow::<SharedDatabase>();
    let schema = state.borrow::<Arc<SchemaBuilder>>();

    let handle = Bundle::new(
        &*db.read(),
        b.name,
        b.items
            .into_iter()
            .map(|i| BundleItem {
                prefix: i.prefix,
                content: map_js_dep(i.content),
            })
            .collect(),
    );

    schema.register_bundle(handle);

    Ok(usize::from(kabina_db::AsId::as_id(handle)) as f64)
}
