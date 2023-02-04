use std::path::PathBuf;
use std::rc::Rc;

use deno_core::op;
use deno_core::Extension;
use deno_core::JsRuntime;
use deno_core::RuntimeOptions;
use module::KabinaModuleLoader;

mod module;

#[op]
fn op_sum(nums: Vec<f64>) -> Result<f64, deno_core::error::AnyError> {
    // Sum inputs
    let sum = nums.iter().fold(0.0, |a, v| a + v);
    // return as a Result<f64, AnyError>
    Ok(sum)
}

pub async fn invoke(schema: PathBuf) {
    // Build a deno_core::Extension providing custom ops
    let ext = Extension::builder("my_ext")
        .ops(vec![
            // An op for summing an array of numbers
            // The op-layer automatically deserializes inputs
            // and serializes the returned Result & value
            op_sum::decl(),
        ])
        .build();

    // Initialize a runtime instance
    let mut runtime = JsRuntime::new(RuntimeOptions {
        module_loader: Some(Rc::new(KabinaModuleLoader)),

        extensions: vec![ext],
        ..Default::default()
    });

    let url = deno_core::url::Url::from_file_path(&schema).unwrap();
    let source = tokio::fs::read_to_string(schema).await.unwrap();
    let module = runtime.load_main_module(&url, Some(source)).await.unwrap();

    // runtime.execute_script("<schema>", &schema).unwrap();
}
