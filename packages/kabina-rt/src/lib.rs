#![feature(async_fn_in_trait)]

use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use deno_core::Extension;
use deno_core::JsRuntime;
use deno_core::RuntimeOptions;
use kabina_db::runtime::Runtime;
use kabina_db::AsId;
use kabina_db::Schema;
use kabina_db::SchemaBuilder;
use kabina_db::SharedDatabase;
use module::KabinaModuleLoader;

mod collection;
mod fileset;
mod module;
mod server;
mod toolchain;
mod transform;

pub struct DenoRuntime {
    db: SharedDatabase,
    runtime: JsRuntime,
}

impl DenoRuntime {
    pub fn new(db: SharedDatabase) -> Self {
        let ext = Extension::builder("runtime")
            .ops(vec![fileset::file_group::decl()])
            .ops(vec![transform::transform::decl()])
            .ops(vec![collection::collection::decl()])
            .ops(vec![server::server::decl()])
            .ops(vec![toolchain::toolchain::decl()])
            .build();

        // Initialize a runtime instance
        let mut runtime = JsRuntime::new(RuntimeOptions {
            module_loader: Some(Rc::new(KabinaModuleLoader)),
            extensions: vec![ext],
            ..Default::default()
        });

        runtime.op_state().borrow_mut().put(db.clone());

        DenoRuntime { db, runtime }
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
        let value = self
            .runtime
            .execute_script("<transform>", &format!("__transforms.get({})()", id))
            .unwrap();

        let value = value.open(self.runtime.v8_isolate());

        println!("{:?}", value.is_boolean());

        todo!();
    }
}
