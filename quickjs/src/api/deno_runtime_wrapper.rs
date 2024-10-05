use super::{
    deno_extension,
    deno_helper::*,
    deno_loader::{DenoLoader, DenoLoaderState},
};
use crate::api::deno_commons::DenoMemoryUsage;
use deno_core::{anyhow::anyhow, JsRuntime};
use deno_core::{futures::TryFutureExt, v8};
use flutter_rust_bridge::frb;
use serde::Serialize;
use std::sync::{Arc, Mutex};

#[frb(external)]
#[frb(opaque)]
pub(crate) struct DenoRuntime {
    pub runtime: JsRuntime,
    pub sys_module: Option<(usize, String)>,
    pub loader: Arc<Mutex<DenoLoaderState>>,
    pub tokio: Arc<tokio::runtime::Runtime>,
}
// Ensure that Wrapper implements Sync
unsafe impl Send for DenoRuntime {}
unsafe impl Sync for DenoRuntime {}
#[derive(Clone)]
struct Permissions;

impl deno_web::TimersPermission for Permissions {
    fn allow_hrtime(&mut self) -> bool {
        false
    }
}
impl deno_fetch::FetchPermissions for Permissions {
    fn check_net_url(
        &mut self,
        _url: &deno_core::url::Url,
        _api_name: &str,
    ) -> Result<(), deno_core::error::AnyError> {
        Err(anyhow!("check_fetch_url_forbidden"))
    }

    fn check_read(
        &mut self,
        _p: &std::path::Path,
        _api_name: &str,
    ) -> Result<(), deno_core::error::AnyError> {
        Err(anyhow!("check_fetch_read_forbidden"))
    }
}
impl deno_net::NetPermissions for Permissions {
    fn check_net<T: AsRef<str>>(
        &mut self,
        _host: &(T, Option<u16>),
        _api_name: &str,
    ) -> Result<(), deno_core::error::AnyError> {
        Err(anyhow!("check_net_forbidden"))
    }

    fn check_read(
        &mut self,
        _p: &std::path::Path,
        _api_name: &str,
    ) -> Result<(), deno_core::error::AnyError> {
        Err(anyhow!("check_net_read_forbidden"))
    }

    fn check_write(
        &mut self,
        _p: &std::path::Path,
        _api_name: &str,
    ) -> Result<(), deno_core::error::AnyError> {
        Err(anyhow!("check_net_write_forbidden"))
    }
}

#[derive(Clone, Debug)]
pub(crate) struct DenoRuntimeOptions {
    pub snapshot: Option<&'static [u8]>,
    pub tokio: Arc<tokio::runtime::Runtime>,
}

impl DenoRuntime {
    pub fn new(options: DenoRuntimeOptions) -> Result<Self, String> {
        let extensions = vec![
            deno_webidl::deno_webidl::init_ops_and_esm(),
            deno_console::deno_console::init_ops_and_esm(),
            deno_url::deno_url::init_ops_and_esm(),
            deno_web::deno_web::init_ops_and_esm::<Permissions>(
                Default::default(),
                Default::default(),
            ),
            deno_fetch::deno_fetch::init_ops_and_esm::<Permissions>(Default::default()),
            deno_net::deno_net::init_ops_and_esm::<Permissions>(
                Default::default(),
                Default::default(),
            ),
            //deno_runtime::runtime::init_ops_and_esm(),
            //deno_runtime::ops::worker_host::deno_worker_host::init_ops_and_esm(),
            //deno_runtime::ops::permissions::deno_permissions::init_ops_and_esm(),
            //deno_runtime::ops::bootstrap::deno_bootstrap::init_ops_and_esm(None),
            deno_extension::custom_extension::init_ops_and_esm(),
        ];
        // init v8
        deno_core::v8_set_flags(vec![
            "--no-strict".to_string(),
            // default v8 984 (kB)
            "--stack-size=98400".to_string(),
        ]);
        deno_core::JsRuntime::init_platform(None, true);
        let loader_inner = Arc::new(Mutex::new(DenoLoaderState::new()));
        // build using snapshot
        if let Some(snapshot) = options.snapshot {
            let runtime = deno_core::JsRuntime::try_new(deno_core::RuntimeOptions {
                startup_snapshot: Some(snapshot),
                extensions,
                is_main: true,
                module_loader: Some(std::rc::Rc::new(DenoLoader::new(loader_inner.clone()))),
                ..Default::default()
            })
            .map_err(|e| e.to_string())?;
            return Self::from_runtime(runtime, loader_inner, options.tokio);
        }
        // without snapshot
        let runtime = deno_core::JsRuntime::try_new(deno_core::RuntimeOptions {
            startup_snapshot: None,
            extensions,
            is_main: true,
            module_loader: Some(std::rc::Rc::new(DenoLoader::new(loader_inner.clone()))),
            ..Default::default()
        })
        .map_err(|e| e.to_string())?;
        return Self::from_runtime(runtime, loader_inner, options.tokio);
    }

    pub fn from_runtime(
        runtime: deno_core::JsRuntime,
        loader: Arc<Mutex<DenoLoaderState>>,
        tokio: Arc<tokio::runtime::Runtime>,
    ) -> Result<Self, String> {
        let object = Self {
            runtime,
            tokio,
            sys_module: (None),
            loader: loader.clone(),
        };
        Ok(object)
    }
    pub fn bootstrap(&mut self) -> Result<(), String> {
        let context = self.runtime.main_context();
        let mut scope = self.runtime.handle_scope();
        let scope = &mut v8::TryCatch::new(&mut scope);
        let context_local = v8::Local::new(scope, context);
        let global_obj = context_local.global(scope);
        let bootstrap_str = v8::String::new_external_onebyte_static(scope, b"bootstrap")
            .ok_or("bootstrap_name_error")?;
        let bootstrap_ns: v8::Local<v8::Value> = global_obj
            .get(scope, bootstrap_str.into())
            .ok_or("bootstrap_ns_notfound")?;
        let bootstrap_ns = v8::Local::<v8::Object>::try_from(bootstrap_ns);
        let bootstrap_ns = bootstrap_ns.map_err(|e: deno_core::v8::DataError| e.to_string())?;
        //call bootstrap https://github.com/denoland/deno/blob/main/runtime/worker_bootstrap.rs#L94
        let main_runtime_str = v8::String::new_external_onebyte_static(scope, b"mainRuntime")
            .ok_or("main_runtime_error")?;
        let main_runtime_str: v8::Local<v8::Value> = main_runtime_str.into();
        let bootstrap_fn = bootstrap_ns.get(scope, main_runtime_str);
        let bootstrap_fn = bootstrap_fn.ok_or("bootstrap_to_local_fail")?;
        let undefined = v8::undefined(scope);
        let bootstrap_fn =
            v8::Local::<v8::Function>::try_from(bootstrap_fn).map_err(|e| e.to_string())?;
        v8::Global::new(scope, bootstrap_fn);
        let args = BootstrapOptions {
            mode: WorkerExecutionMode::Run,
            ..Default::default()
        };
        let args = args.as_v8(scope)?;
        bootstrap_fn.call(scope, undefined.into(), &[args]);
        if let Some(exception) = scope.exception() {
            let error = deno_core::error::JsError::from_v8_exception(scope, exception);
            return Err(format!("bootstrap_error {}", error.to_string()));
        }
        Ok(())
    }

    pub fn interrupt(&mut self) -> Result<bool, String> {
        //extern "C" fn handle_interrupt(_isolate: &mut deno_core::v8::Isolate, _arg: *mut std::ffi::c_void) {}
        //let data: *mut std::ffi::c_void = std::ptr::null_mut();
        //let res = unwrapped
        //    .runtime
        //    .v8_isolate()
        //    .thread_safe_handle()
        //    .request_interrupt(handle_interrupt, data);
        //let res = scope.terminate_execution();
        //let is_terminating = unwrapped.runtime.v8_isolate().is_execution_terminating();
        self.spawn_in_scope(move |scope| {
            let str = "interrupted".to_v8_string(scope).map_err(|e| e.to_string());
            if let Ok(str) = str {
                let exc = deno_core::v8::Exception::error(scope, str);
                scope.throw_exception(exc);
            }
            let handle = scope.thread_safe_handle();
            handle.terminate_execution();
        });
        let res = self.runtime.v8_isolate().terminate_execution();
        Ok(res)
    }

    pub fn spawn_in_scope<F>(&mut self, f: F)
    where
        F: FnOnce(&mut v8::HandleScope) + Send + 'static,
    {
        self.runtime
            .op_state()
            .borrow()
            .borrow::<deno_core::V8CrossThreadTaskSpawner>()
            .spawn(f);
    }

    pub fn get_memory_usage(&mut self) -> Result<DenoMemoryUsage, String> {
        let mut s = deno_core::v8::HeapStatistics::default();
        let scope = &mut deno_core::v8::HandleScope::new(self.runtime.v8_isolate());
        scope.get_heap_statistics(&mut s);
        let res = DenoMemoryUsage {
            total_available_size: s.total_available_size(),
            total_physical_size: s.total_available_size(),
            total_heap_size: s.total_heap_size(),
            used_heap_size: s.used_heap_size(),
            external_memory: s.external_memory(),
        };
        Ok(res)
    }
    pub async fn execute_module(
        &mut self,
        module_name: String,
        module_code: String,
        is_main: bool,
    ) -> Result<usize, String> {
        if is_main {
            self.execute_main_module(module_name, module_code).await
        } else {
            self.execute_side_module(module_name, module_code).await
        }
    }

    pub async fn execute_side_module(
        &mut self,
        module_name: String,
        module_code: String,
    ) -> Result<usize, String> {
        let id = self.preload_side_module(module_name, module_code).await?;
        self.evaluate_module(id).await.map_err(|e| e.to_string())?;
        Ok(id)
    }

    pub async fn execute_main_module(
        &mut self,
        module_name: String,
        module_code: String,
    ) -> Result<usize, String> {
        let id = self.preload_main_module(module_name, module_code).await?;
        self.evaluate_module(id).await.map_err(|e| e.to_string())?;
        Ok(id)
    }

    pub async fn preload_module(
        &mut self,
        module_name: String,
        module_code: String,
        is_main: bool,
    ) -> Result<usize, String> {
        if is_main {
            self.preload_main_module(module_name, module_code).await
        } else {
            self.preload_side_module(module_name, module_code).await
        }
    }

    pub async fn preload_main_module(
        &mut self,
        module_name: String,
        module_code: String,
    ) -> Result<usize, String> {
        // add to loader
        {
            let mut loader = self.loader.lock().map_err(|e| e.to_string())?;
            let _ = loader.add_allowed(module_name.clone(), module_code, None)?;
        }
        // compute specifier
        let module_specifier = DenoLoaderState::to_specifier(&module_name)
            .map_err(|_| format!("invalid_mod_name {}", module_name))?;
        // load
        let id = self
            .runtime
            .load_main_es_module(&module_specifier)
            .map_err(|e| e.to_string())
            .await?;
        Ok(id)
    }

    pub async fn preload_side_module(
        &mut self,
        module_name: String,
        module_code: String,
    ) -> Result<usize, String> {
        // add to loader
        {
            let mut loader = self.loader.lock().map_err(|e| e.to_string())?;
            let _ = loader.add_allowed(module_name.clone(), module_code, None)?;
        }
        // compute specifier
        let module_specifier = DenoLoaderState::to_specifier(&module_name)
            .map_err(|_| format!("invalid_mod_name {}", module_name))?;
        // load
        let id = self
            .runtime
            .load_side_es_module(&module_specifier)
            .map_err(|e| e.to_string())
            .await?;
        Ok(id)
    }

    pub async fn evaluate_module(&mut self, id: usize) -> Result<(), String> {
        let mut receiver = self.runtime.mod_evaluate(id);
        tokio::select! {
          // Not using biased mode leads to non-determinism for relatively simple
          // programs.
          biased;

          maybe_result = &mut receiver => {
            maybe_result.map_err(|e|e.to_string())
          }

          event_loop_result = self.run_event_loop(false) => {
            event_loop_result?;
            receiver.map_err(|e|e.to_string()).await
          }
        }
    }

    pub async fn run_event_loop(&mut self, wait_for_inspector: bool) -> Result<(), String> {
        self.runtime
            .run_event_loop(deno_core::PollEventLoopOptions {
                wait_for_inspector,
                ..Default::default()
            })
            .map_err(|e| e.to_string())
            .await
    }

    pub async fn execute_sys_module(
        &mut self,
        module_name: String,
        module_code: String,
        evaluate: bool,
    ) -> Result<usize, String> {
        if evaluate {
            let id = self
                .execute_side_module(module_name.clone(), module_code)
                .await?;
            self.sys_module = Some((id, module_name));
            Ok(id)
        } else {
            let id = self
                .preload_side_module(module_name.clone(), module_code)
                .await?;
            self.sys_module = Some((id, module_name));
            Ok(id)
        }
    }

    pub async fn call_module_function_to_json(
        &mut self,
        module: (usize, String),
        function: String,
        args: &impl serde::ser::Serialize,
    ) -> Result<String, String> {
        let runtime = &mut self.runtime;
        return deno_call_function_to_json(runtime, Some(module), function, &args).await;
    }

    pub async fn call_global_function_to_json(
        &mut self,
        function: String,
        args: &impl serde::ser::Serialize,
    ) -> Result<String, String> {
        let runtime = &mut self.runtime;
        return deno_call_function_to_json(runtime, None, function, &args).await;
    }

    pub fn eval_to_json(&mut self, code: String) -> Result<String, String> {
        let result = self
            .runtime
            .execute_script("", code)
            .map_err(|e| e.to_string())?;
        let scope = &mut self.runtime.handle_scope();
        return deno_value_to_json(scope, result);
    }
}

/*
///
/// Arguments
///
#[derive(Clone, Debug)]
pub(crate) struct DenoArguments {
    arguments: Vec<DenoArgument>,
}

impl DenoArguments {
    pub fn new() -> Self {
        Self {
            arguments: Vec::new(),
        }
    }
    pub fn push(&mut self, arg: DenoArgument) -> &mut Self {
        self.arguments.push(arg);
        self
    }
}
unsafe impl Send for DenoArguments {}
unsafe impl Sync for DenoArguments {}

impl serde::ser::Serialize for DenoArguments {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_seq(self.arguments.clone())
    }
}
///
/// Argument
///
#[derive(Clone, Debug)]
pub(crate) struct DenoArgument {
    binary: Option<Vec<u8>>,
    json: Option<String>,
}
impl DenoArgument {
    pub fn from_binary(bin: Vec<u8>) -> Self {
        return Self {
            binary: Some(bin),
            json: None,
        };
    }
    pub fn from_json(json: String) -> Self {
        return Self {
            binary: None,
            json: Some(json),
        };
    }
}
unsafe impl Send for DenoArgument {}
unsafe impl Sync for DenoArgument {}

impl serde::ser::Serialize for DenoArgument {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if let Some(binary) = &self.binary {
            return serializer.collect_seq(binary);
        }
        if let Some(json) = &self.json {
            return serializer.serialize_str(json.as_str());
        }
        return serializer.serialize_none();
    }
}
 */
#[derive(Debug, Default, Clone, Copy)]
pub enum WorkerLogLevel {
    // WARNING: Ensure this is kept in sync with
    // the JS values (search for LogLevel).
    Error = 1,
    Warn = 2,
    #[default]
    Info = 3,
    Debug = 4,
}
#[derive(Debug, Default, Clone, Copy)]
pub enum ColorLevel {
    #[default]
    None = 1,
}
#[derive(Copy, Clone)]
pub enum WorkerExecutionMode {
    /// No special behaviour.
    None,

    /// Running in a worker.
    Worker,
    /// `deno run`
    Run,
    /// `deno repl`
    Repl,
    /// `deno eval`
    Eval,
    /// `deno test`
    Test,
    /// `deno bench`
    Bench,
    /// `deno serve`
    Serve {
        is_main: bool,
        worker_count: Option<usize>,
    },
    /// `deno jupyter`
    Jupyter,
}
impl WorkerExecutionMode {
    pub fn discriminant(&self) -> u8 {
        match self {
            WorkerExecutionMode::None => 0,
            WorkerExecutionMode::Worker => 1,
            WorkerExecutionMode::Run => 2,
            WorkerExecutionMode::Repl => 3,
            WorkerExecutionMode::Eval => 4,
            WorkerExecutionMode::Test => 5,
            WorkerExecutionMode::Bench => 6,
            WorkerExecutionMode::Serve { .. } => 7,
            WorkerExecutionMode::Jupyter => 8,
        }
    }
    pub fn serve_info(&self) -> (Option<bool>, Option<usize>) {
        match *self {
            WorkerExecutionMode::Serve {
                is_main,
                worker_count,
            } => (Some(is_main), worker_count),
            _ => (None, None),
        }
    }
}

#[derive(Clone)]
pub struct BootstrapOptions {
    pub deno_version: String,
    /// Sets `Deno.args` in JS runtime.
    pub args: Vec<String>,
    pub cpu_count: usize,
    pub log_level: WorkerLogLevel,
    pub enable_op_summary_metrics: bool,
    pub enable_testing_features: bool,
    pub locale: String,
    pub location: Option<deno_core::ModuleSpecifier>,
    /// Sets `Deno.noColor` in JS runtime.
    pub no_color: bool,
    pub is_stdout_tty: bool,
    pub is_stderr_tty: bool,
    pub color_level: ColorLevel,
    // --unstable flag, deprecated
    pub unstable: bool,
    // --unstable-* flags
    pub unstable_features: Vec<i32>,
    pub user_agent: String,
    pub inspect: bool,
    pub has_node_modules_dir: bool,
    pub argv0: Option<String>,
    pub node_debug: Option<String>,
    pub node_ipc_fd: Option<i64>,
    pub disable_deprecated_api_warning: bool,
    pub verbose_deprecated_api_warning: bool,
    pub future: bool,
    pub mode: WorkerExecutionMode,
    // Used by `deno serve`
    pub serve_port: Option<u16>,
    pub serve_host: Option<String>,
}

impl Default for BootstrapOptions {
    fn default() -> Self {
        let cpu_count = 1;

        let runtime_version = env!("CARGO_PKG_VERSION");
        let user_agent = format!("Deno/{runtime_version}");

        Self {
            deno_version: runtime_version.to_string(),
            user_agent,
            cpu_count,
            no_color: true,
            is_stdout_tty: false,
            is_stderr_tty: false,
            color_level: ColorLevel::None,
            enable_op_summary_metrics: Default::default(),
            enable_testing_features: Default::default(),
            log_level: Default::default(),
            locale: "en".to_string(),
            location: Default::default(),
            unstable: Default::default(),
            unstable_features: Default::default(),
            inspect: Default::default(),
            args: Default::default(),
            has_node_modules_dir: Default::default(),
            argv0: None,
            node_debug: None,
            node_ipc_fd: None,
            disable_deprecated_api_warning: false,
            verbose_deprecated_api_warning: false,
            future: false,
            mode: WorkerExecutionMode::None,
            serve_port: Default::default(),
            serve_host: Default::default(),
        }
    }
}

/// This is a struct that we use to serialize the contents of the `BootstrapOptions`
/// struct above to a V8 form. While `serde_v8` is not as fast as hand-coding this,
/// it's "fast enough" while serializing a large tuple like this that it doesn't appear
/// on flamegraphs.
///
/// Note that a few fields in here are derived from the process and environment and
/// are not sourced from the underlying `BootstrapOptions`.
///
/// Keep this in sync with `99_main.js`.
#[derive(serde::Serialize)]
struct BootstrapV8<'a>(
    // deno version
    &'a str,
    // location
    Option<&'a str>,
    // unstable
    bool,
    // granular unstable flags
    &'a [i32],
    // inspect
    bool,
    // enable_testing_features
    bool,
    // has_node_modules_dir
    bool,
    // argv0
    Option<&'a str>,
    // node_debug
    Option<&'a str>,
    // disable_deprecated_api_warning,
    bool,
    // verbose_deprecated_api_warning
    bool,
    // future
    bool,
    // mode
    i32,
    // serve port
    u16,
    // serve host
    Option<&'a str>,
    // serve is main
    Option<bool>,
    // serve worker count
    Option<usize>,
);

impl BootstrapOptions {
    /// Return the v8 equivalent of this structure.
    pub fn as_v8<'s>(
        &self,
        scope: &mut v8::HandleScope<'s>,
    ) -> Result<v8::Local<'s, v8::Value>, String> {
        let scope = std::cell::RefCell::new(scope);
        let ser = deno_core::serde_v8::Serializer::new(&scope);
        let res = self.serialize(ser).map_err(|e| e.to_string());
        res
    }
}

impl serde::Serialize for BootstrapOptions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let (serve_is_main, serve_worker_count) = self.mode.serve_info();
        let bootstrap = BootstrapV8(
            &self.deno_version,
            self.location.as_ref().map(|l| l.as_str()),
            self.unstable,
            self.unstable_features.as_ref(),
            self.inspect,
            self.enable_testing_features,
            self.has_node_modules_dir,
            self.argv0.as_deref(),
            self.node_debug.as_deref(),
            self.disable_deprecated_api_warning,
            self.verbose_deprecated_api_warning,
            self.future,
            self.mode.discriminant() as _,
            self.serve_port.unwrap_or_default(),
            self.serve_host.as_deref(),
            serve_is_main,
            serve_worker_count,
        );
        let res = bootstrap.serialize(serializer)?;
        Ok(res)
    }
}
