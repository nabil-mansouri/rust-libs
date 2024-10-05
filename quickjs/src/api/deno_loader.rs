use std::sync::Arc;

use deno_core::{anyhow::anyhow, url::Url, FastString, ModuleLoader, ModuleSource, ModuleType};

#[derive(Debug)]
pub(crate) struct ModuleInfo {
    name: String,
    path: String,
    code: String,
    _id: Option<usize>,
}

pub(crate) struct DenoLoaderState {
    modules: Vec<std::sync::Arc<ModuleInfo>>,
}
impl DenoLoaderState {
    pub fn to_specifier(mod_name: &str) -> Result<Url, String> {
        deno_core::resolve_path(mod_name, std::path::Path::new("/")).map_err(|e| e.to_string())
    }
    pub fn new() -> Self {
        return Self {
            modules: Vec::new(),
        };
    }

    pub fn add_allowed(
        &mut self,
        mod_name: String,
        mod_code: String,
        modid: Option<usize>,
    ) -> Result<(), String> {
        let path = deno_core::resolve_path(mod_name.as_str(), std::path::Path::new("/"))
            .map_err(|e| e.to_string())?;
        self.modules.push(std::sync::Arc::new(ModuleInfo {
            name: mod_name,
            path: path.to_string(),
            code: mod_code,
            _id: modid,
        }));
        Ok(())
    }
}
pub(crate) struct DenoLoader {
    inner: std::sync::Arc<std::sync::Mutex<DenoLoaderState>>,
}
impl DenoLoader {
    pub fn new(inner: std::sync::Arc<std::sync::Mutex<DenoLoaderState>>) -> Self {
        return Self { inner };
    }

    pub fn search_by_name_or_path(&self, name_or_path: String) -> Option<Arc<ModuleInfo>> {
        if let Ok(mutex) = self.inner.lock() {
            for info in mutex.modules.iter() {
                if info.name.eq(name_or_path.as_str()) || info.path.eq(name_or_path.as_str()) {
                    return Some(info.to_owned());
                }
            }
        }
        return None;
    }
}
impl ModuleLoader for DenoLoader {
    fn resolve(
        &self,
        specifier: &str,
        _referrer: &str,
        _kind: deno_core::ResolutionKind,
    ) -> Result<deno_core::ModuleSpecifier, deno_core::anyhow::Error> {
        if specifier.starts_with("file:/") {
            let url = Url::parse(specifier).map_err(|e| anyhow!(e.to_string()))?;
            return Ok(url);
        }
        let res = DenoLoaderState::to_specifier(specifier).map_err(|e| anyhow!(e))?;
        Ok(res)
    }

    fn load(
        &self,
        module_specifier: &deno_core::ModuleSpecifier,
        _maybe_referrer: Option<&deno_core::ModuleSpecifier>,
        _is_dyn_import: bool,
        _requested_module_type: deno_core::RequestedModuleType,
    ) -> deno_core::ModuleLoadResponse {
        if let Some(module) = self.search_by_name_or_path(module_specifier.to_string()) {
            let source_code = deno_core::ModuleSourceCode::String(FastString::from(module.code.clone()));
            deno_core::ModuleLoadResponse::Sync(Ok(ModuleSource::new(
                ModuleType::JavaScript,
                source_code,
                module_specifier,
                None,
            )))
        } else {
            deno_core::ModuleLoadResponse::Sync(Err(anyhow!(
                "module_not_allowed {}",
                module_specifier.as_str()
            )))
        }
    }
}
