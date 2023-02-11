use std::sync::Arc;

use deno_core::{op, OpState};
use kabina_db::{AsId, Database, DependencyKind, FileGroup, RunnerKind, SchemaBuilder, Transform};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(tag = "kind")]
pub enum JsDependency {
    FileGroup { id: usize },
    Transform { id: usize },
}

#[derive(Deserialize)]
pub struct JsTransform {
    name: String,
    module: deno_core::url::Url,
    runner: u64,
    input: Vec<JsDependency>,
    dependencies: Vec<JsDependency>,
}

fn map_js_dep(dep: JsDependency) -> DependencyKind {
    match dep {
        JsDependency::FileGroup { id } => DependencyKind::FileGroup(FileGroup::from_id(id.into())),
        JsDependency::Transform { id } => DependencyKind::Transform(Transform::from_id(id.into())),
    }
}

#[op]
pub fn transform(state: &mut OpState, f: JsTransform) -> Result<f64, deno_core::error::AnyError> {
    let mut root = f.module.to_file_path().unwrap();
    if root.extension().is_some() {
        root = root.parent().unwrap().to_owned()
    }

    tracing::info!("Transform {:?} created at {:?}", f.name, root.to_str());

    let db = state.borrow::<Arc<Database>>();
    let schema = state.borrow::<Arc<SchemaBuilder>>();

    let input = f.input.into_iter().map(map_js_dep).collect();
    let dependencies = f.dependencies.into_iter().map(map_js_dep).collect();

    let handle = Transform::new(
        &**db,
        f.name,
        RunnerKind::JsFunction(f.runner),
        input,
        dependencies,
    );

    schema.register_transform(handle);

    Ok(usize::from(kabina_db::AsId::as_id(handle)) as f64)
}
