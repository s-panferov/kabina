use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use deno_core::op;
use deno_core::serde::Deserialize;
use deno_core::Extension;
use deno_core::JsRuntime;
use deno_core::OpState;
use deno_core::RuntimeOptions;
use kabina_db::Database;
use kabina_db::Schema;
use kabina_db::SchemaBuilder;
use module::KabinaModuleLoader;

mod module;

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
fn file_group(state: &mut OpState, f: JsFileGroup) -> Result<f64, deno_core::error::AnyError> {
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

    let db = state.borrow::<Arc<Database>>();
    let schema = state.borrow::<Arc<SchemaBuilder>>();

    let handle = kabina_db::FileGroup::new(
        &**db,
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

pub async fn invoke(schema_path: PathBuf, database: Arc<kabina_db::Database>) -> Schema {
    let ext = Extension::builder("runtime")
        .ops(vec![file_group::decl()])
        .build();

    // Initialize a runtime instance
    let mut runtime = JsRuntime::new(RuntimeOptions {
        module_loader: Some(Rc::new(KabinaModuleLoader)),
        extensions: vec![ext],
        ..Default::default()
    });

    let schema = Arc::new(SchemaBuilder::default());

    runtime.op_state().borrow_mut().put(database.clone());
    runtime.op_state().borrow_mut().put(schema);

    let url = deno_core::url::Url::from_file_path(&schema_path).unwrap();
    let source = tokio::fs::read_to_string(schema_path).await.unwrap();
    let module = runtime.load_main_module(&url, Some(source)).await.unwrap();

    let eval = runtime.mod_evaluate(module);

    runtime.run_event_loop(false).await.unwrap();
    eval.await.unwrap().unwrap();

    let builder = runtime.op_state().borrow_mut().take::<Arc<SchemaBuilder>>();
    let Ok(builder) = Arc::try_unwrap(builder) else {
        panic!("Arc cloned")
    };

    Schema::new(&*database, builder.file_groups)

    // runtime.execute_script("<schema>", &schema).unwrap();
}
