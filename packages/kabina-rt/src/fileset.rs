use std::sync::Arc;

use deno_core::{op, OpState};
use kabina_db::{SchemaBuilder, SharedDatabase};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum JsFileGroupStategy {
    Hash,
    Time,
}

#[derive(Debug, Deserialize)]
pub struct JsFileGroupItem {
    strategy: Option<JsFileGroupStategy>,
    pattern: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum JsFileGroupItemShortcut {
    String(String),
    Item(JsFileGroupItem),
}

#[derive(Deserialize)]
pub struct JsFileGroup {
    name: String,
    module: deno_core::url::Url,
    root: Option<String>,
    items: Vec<JsFileGroupItemShortcut>,
}

#[op]
pub fn file_group(state: &mut OpState, f: JsFileGroup) -> Result<f64, deno_core::error::AnyError> {
    let mut module_root = f.module.to_file_path().unwrap();
    if module_root.extension().is_some() {
        module_root = module_root.parent().unwrap().to_owned()
    }

    let root = if let Some(root) = f.root {
        module_root.join(root)
    } else {
        module_root
    };

    tracing::info!("File group {:?} created at {:?}", f.name, root.to_str());

    let db = state.borrow::<SharedDatabase>();
    let schema = state.borrow::<Arc<SchemaBuilder>>();

    let handle = kabina_db::FileGroup::new(
        &*db.read(),
        f.name,
        root,
        f.items
            .into_iter()
            .map(|i| match i {
                JsFileGroupItemShortcut::String(s) => kabina_db::FileGroupItem {
                    pattern: s,
                    strategy: kabina_db::FileGroupStategy::Time,
                },
                JsFileGroupItemShortcut::Item(i) => kabina_db::FileGroupItem {
                    strategy: match i.strategy {
                        Some(JsFileGroupStategy::Hash) => kabina_db::FileGroupStategy::Hash,
                        Some(JsFileGroupStategy::Time) => kabina_db::FileGroupStategy::Time,
                        _ => kabina_db::FileGroupStategy::Time,
                    },
                    pattern: i.pattern,
                },
            })
            .collect(),
    );

    schema.register_file_group(handle);

    Ok(usize::from(kabina_db::AsId::as_id(handle)) as f64)
}
