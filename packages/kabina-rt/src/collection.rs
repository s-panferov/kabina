use std::{path::PathBuf, sync::Arc};

use deno_core::{op, OpState};
use kabina_db::{Collection, CollectionItem, SchemaBuilder, SharedDatabase};
use serde::Deserialize;

use crate::transform::{map_js_dep, JsDependency};

#[derive(Deserialize)]
pub struct JsCollectionItem {
    prefix: PathBuf,
    content: JsDependency,
}

#[derive(Deserialize)]
pub struct JsCollection {
    name: String,
    items: Vec<JsCollectionItem>,
}

#[op]
pub fn collection(state: &mut OpState, b: JsCollection) -> Result<f64, deno_core::error::AnyError> {
    tracing::info!("Collection {:?} created ", b.name);

    let db = state.borrow::<SharedDatabase>();
    let schema = state.borrow::<Arc<SchemaBuilder>>();

    let handle = Collection::new(
        &*db.read(),
        b.name,
        b.items
            .into_iter()
            .map(|i| CollectionItem {
                prefix: i.prefix,
                content: map_js_dep(i.content)
                    .to_input_kind()
                    .expect("Input dependency"),
            })
            .collect(),
    );

    schema.register_collection(handle);

    Ok(usize::from(kabina_db::AsId::as_id(handle)) as f64)
}
