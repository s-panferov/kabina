use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use deno_core::Extension;
use deno_core::JsRuntime;
use deno_core::RuntimeOptions;
use kabina_db::Schema;
use kabina_db::SchemaBuilder;
use module::KabinaModuleLoader;

mod fileset;
mod module;
mod transform;

pub async fn invoke(schema_path: PathBuf, database: Arc<kabina_db::Database>) -> Schema {
    let ext = Extension::builder("runtime")
        .ops(vec![fileset::file_group::decl()])
        .ops(vec![transform::transform::decl()])
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
