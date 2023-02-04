use std::path::PathBuf;
use std::rc::Rc;

use deno_core::op;
use deno_core::serde::Deserialize;
use deno_core::Extension;
use deno_core::JsRuntime;
use deno_core::RuntimeOptions;
use module::KabinaModuleLoader;

mod module;

#[derive(Deserialize)]
pub struct FileGroup {
    name: Option<String>,
}

#[op]
fn file_group(f: FileGroup) -> Result<f64, deno_core::error::AnyError> {
    tracing::info!("File group created");
    Ok(0f64)
}

pub async fn invoke(schema: PathBuf) {
    let ext = Extension::builder("runtime")
        .ops(vec![file_group::decl()])
        .build();

    // Initialize a runtime instance
    let mut runtime = JsRuntime::new(RuntimeOptions {
        module_loader: Some(Rc::new(KabinaModuleLoader)),
        extensions: vec![ext],
        ..Default::default()
    });

    runtime.op_state().borrow_mut();

    let url = deno_core::url::Url::from_file_path(&schema).unwrap();
    let source = tokio::fs::read_to_string(schema).await.unwrap();
    let module = runtime.load_main_module(&url, Some(source)).await.unwrap();

    let eval = runtime.mod_evaluate(module);

    runtime.run_event_loop(false).await.unwrap();
    eval.await.unwrap().unwrap();

    // runtime.execute_script("<schema>", &schema).unwrap();
}
