use std::pin::Pin;
use std::str::FromStr;

use anyhow::anyhow;
use anyhow::bail;
use anyhow::Error;
use deno_ast::MediaType;
use deno_ast::ParseParams;
use deno_ast::SourceTextInfo;
use deno_core::resolve_import;
use deno_core::ModuleCode;
use deno_core::ModuleLoader;
use deno_core::ModuleSource;
use deno_core::ModuleSourceFuture;
use deno_core::ModuleSpecifier;
use deno_core::ModuleType;
use deno_core::ResolutionKind;
use futures::FutureExt;
pub struct KabinaModuleLoader;

pub const RUNTIME_URL: &'static str = "kabina:///runtime.ts";
pub const RUNTIME: &'static str = include_str!("../runtime.ts");

impl KabinaModuleLoader {
    pub fn runtime_module_specifier() -> ModuleSpecifier {
        ModuleSpecifier::from_str(RUNTIME_URL).unwrap()
    }
}

impl ModuleLoader for KabinaModuleLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        _kind: ResolutionKind,
    ) -> Result<ModuleSpecifier, Error> {
        match specifier {
            "kabina" => return Ok(Self::runtime_module_specifier()),
            _ => {}
        }

        Ok(resolve_import(specifier, referrer)?)
    }

    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        _maybe_referrer: Option<ModuleSpecifier>,
        _is_dyn_import: bool,
    ) -> Pin<Box<ModuleSourceFuture>> {
        let module_specifier = module_specifier.clone();

        async move {
            let path = module_specifier
                .to_file_path()
                .map_err(|_| anyhow!("Only file: URLs are supported."))?;

            let media_type = MediaType::from_path(&path);
            let (module_type, should_transpile) = match media_type {
                MediaType::JavaScript | MediaType::Mjs | MediaType::Cjs => {
                    (ModuleType::JavaScript, false)
                }
                MediaType::Jsx => (ModuleType::JavaScript, true),
                MediaType::TypeScript
                | MediaType::Mts
                | MediaType::Cts
                | MediaType::Dts
                | MediaType::Dmts
                | MediaType::Dcts
                | MediaType::Tsx => (ModuleType::JavaScript, true),
                MediaType::Json => (ModuleType::Json, false),
                _ => bail!("Unknown extension {:?}", path.extension()),
            };

            let code = if module_specifier.as_str() == RUNTIME_URL {
                RUNTIME.to_owned()
            } else {
                tokio::fs::read_to_string(&path).await?
            };

            let code = if should_transpile {
                let parsed = deno_ast::parse_module(ParseParams {
                    specifier: module_specifier.to_string(),
                    text_info: SourceTextInfo::from_string(code),
                    media_type,
                    capture_tokens: false,
                    scope_analysis: false,
                    maybe_syntax: None,
                })?;
                parsed.transpile(&Default::default())?.text
            } else {
                code
            };

            let module = ModuleSource {
                code: ModuleCode::Owned(code.into_bytes()),
                module_type,
                module_url_specified: module_specifier.to_string(),
                module_url_found: module_specifier.to_string(),
            };

            Ok(module)
        }
        .boxed_local()
    }
}
