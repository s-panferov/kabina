#![feature(async_fn_in_trait)]

use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use deno_core::v8;
use deno_core::v8::HandleScope;
use deno_core::v8::Local;
use deno_core::Extension;
use deno_core::JsRuntime;
use deno_core::ModuleLoader;
use deno_core::RuntimeOptions;
use kabina_db::runtime::Runtime;
use kabina_db::AsId;
use kabina_db::Schema;
use kabina_db::SchemaBuilder;
use kabina_db::SharedDatabase;
use module::KabinaModuleLoader;
use module::RUNTIME;

mod collection;
mod fileset;
mod module;
mod server;
mod toolchain;
mod transform;

pub struct DenoRuntime {
    db: SharedDatabase,
    runtime: JsRuntime,
    std: usize,
}

impl DenoRuntime {
    pub async fn new(db: SharedDatabase) -> Self {
        let ext = Extension::builder("runtime")
            .ops(vec![fileset::file_group::decl()])
            .ops(vec![transform::transform::decl()])
            .ops(vec![collection::collection::decl()])
            .ops(vec![server::server::decl()])
            .ops(vec![toolchain::toolchain::decl()])
            .build();

        let loader = Rc::new(KabinaModuleLoader);

        // Initialize a runtime instance
        let mut runtime = JsRuntime::new(RuntimeOptions {
            module_loader: Some(loader.clone()),
            extensions: vec![ext],
            ..Default::default()
        });

        runtime.op_state().borrow_mut().put(db.clone());

        let std_url = &KabinaModuleLoader::runtime_module_specifier();
        let std = runtime
            .load_side_module(
                &std_url,
                Some(
                    String::from_utf8(
                        loader
                            .load(&std_url, None, false)
                            .await
                            .unwrap()
                            .code
                            .to_vec(),
                    )
                    .unwrap(),
                ),
            )
            .await
            .unwrap();

        let receiver = runtime.mod_evaluate(std);
        runtime.run_event_loop(false).await.unwrap();
        let _ = receiver.await;

        DenoRuntime { db, runtime, std }
    }
}

impl Runtime for DenoRuntime {
    async fn load_schema(&mut self, schema_path: PathBuf) -> Schema {
        let schema = Arc::new(SchemaBuilder::default());
        self.runtime.op_state().borrow_mut().put(schema);

        let url = deno_core::url::Url::from_file_path(&schema_path).unwrap();
        let source = tokio::fs::read_to_string(schema_path).await.unwrap();
        let module = self
            .runtime
            .load_main_module(&url, Some(source))
            .await
            .unwrap();

        let eval = self.runtime.mod_evaluate(module);

        self.runtime.run_event_loop(false).await.unwrap();
        eval.await.unwrap().unwrap();

        let builder = self
            .runtime
            .op_state()
            .borrow_mut()
            .take::<Arc<SchemaBuilder>>();

        let Ok(builder) = Arc::try_unwrap(builder) else {
            panic!("Arc cloned")
        };

        Schema::new(
            &*self.db.read(),
            builder.file_groups,
            builder.transforms,
            builder.collections,
            builder.servers,
        )
    }

    async fn transform(&mut self, task: &kabina_db::TransformApply) -> kabina_db::File {
        let id = AsId::as_id(task.transform).as_u32();

        let ns = self.runtime.get_module_namespace(self.std).unwrap();

        let context = self.runtime.global_context();
        let isolate = self.runtime.v8_isolate();

        let ns = ns.open(isolate);
        let mut scope = HandleScope::with_context(isolate, context);

        let string = v8::String::new(&mut scope, "__transforms").unwrap();

        let value = ns.get(&mut scope, string.into()).unwrap();
        let js_id = v8::Number::new(&mut scope, id as f64);

        let function = value
            .to_object(&mut scope)
            .unwrap()
            .get(&mut scope, js_id.into())
            .unwrap();

        let function = Local::<v8::Function>::try_from(function).unwrap();
        let null = v8::null(&mut scope).into();
        let value = function.call(&mut scope, null, &[]).unwrap();

        println!("{:?}", value.is_boolean());

        todo!();
    }
}
