use super::deno_commons::deno_tokio_current_thread;
use super::deno_extension::{
    self, deno_get_extension_callback, deno_set_extension_callback, deno_set_extension_sink,
};
use super::deno_helper::deno_decode_args;
use super::deno_loader::DenoLoader;
use super::deno_runtime_wrapper::{DenoRuntime, DenoRuntimeOptions};
pub use super::wrapper::Wrapper;
use crate::api::deno_commons::DenoCustomData;
use crate::api::deno_commons::DenoMemoryUsage;
use crate::api::deno_loader::DenoLoaderState;
use crate::frb_generated::StreamSink;
use crate::json_args_custom;
use deno_core::futures::TryFutureExt;
pub use std::sync::Arc;
/*
lazy_static::lazy_static! {
    static ref TOKIO_RUNTIME: tokio::runtime::Runtime = {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to build the runtime")
    };
}
*/

static mut SNAPSHOT_LEAK: bool = false;
static mut SNAPSHOT_BYTES: &[u8] = &[];

pub fn deno_set_snapshot(snapshot: Vec<u8>) {
    unsafe {
        // delete previous value
        if SNAPSHOT_LEAK {
            let _ = Box::from_raw(SNAPSHOT_BYTES.as_ptr() as *mut ());
        }
        // set new value
        SNAPSHOT_LEAK = true;
        SNAPSHOT_BYTES = Box::leak(snapshot.into_boxed_slice());
    }
}

pub fn deno_remove_snapshot() -> () {
    deno_set_snapshot(Vec::new());
    ()
}

pub fn deno_get_snapshot() -> Vec<u8> {
    unsafe {
        return SNAPSHOT_BYTES.to_vec();
    }
}

pub fn deno_async_create() -> Result<Arc<Wrapper>, String> {
    let tokio = Arc::new(deno_tokio_current_thread()?);
    let res = tokio
        .clone()
        .block_on(deno_async_create_inner(tokio))
        .map_err(|e| e.to_string());
    res
}
async fn deno_async_create_inner(
    tokio: Arc<tokio::runtime::Runtime>,
) -> Result<Arc<Wrapper>, String> {
    let snapshot = unsafe { SNAPSHOT_BYTES };
    let snapshot_option = if snapshot.len() == 0 {
        None
    } else {
        Some(snapshot)
    };
    let mut runtime = DenoRuntime::new(DenoRuntimeOptions {
        snapshot: snapshot_option,
        tokio,
    })?;
    runtime.bootstrap()?;
    let res_safe = Arc::new(Wrapper::new(runtime));
    Ok(res_safe)
}

pub fn deno_async_interrupt(wrapper: &Arc<Wrapper>) -> Result<bool, String> {
    let unwrapped: &mut DenoRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    unwrapped.interrupt()
}

pub fn deno_async_dispose(wrapper: &Arc<Wrapper>) -> Result<(), String> {
    drop(wrapper.to_owned());
    Ok(())
}

pub fn deno_async_get_memory_usage(wrapper: &Arc<Wrapper>) -> Result<DenoMemoryUsage, String> {
    let unwrapped: &mut DenoRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    unwrapped.get_memory_usage()
}

pub fn deno_async_listen(
    wrapper: &Arc<Wrapper>,
    events: StreamSink<DenoCustomData>,
) -> Result<(), String> {
    let unwrapped: &DenoRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    unwrapped
        .tokio
        .block_on(deno_async_listen_inner(wrapper, events))
}
async fn deno_async_listen_inner(
    wrapper: &Arc<Wrapper>,
    events: StreamSink<DenoCustomData>,
) -> Result<(), String> {
    let unwrapped: &mut DenoRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    let state = unwrapped.runtime.op_state();
    let mut state = state.try_borrow_mut().map_err(|e| e.to_string())?;
    deno_set_extension_sink(&mut state, Arc::new(events));
    deno_set_extension_callback(&mut state);
    Ok(())
}

pub fn deno_async_set_sys_module(
    wrapper: &Arc<Wrapper>,
    module_name: String,
    module_code: Option<String>,
    evaluate: bool,
) -> Result<Option<usize>, String> {
    let unwrapped: &DenoRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    unwrapped
        .tokio
        .block_on(deno_async_set_sys_module_inner(
            wrapper.clone(),
            module_name,
            module_code,
            evaluate,
        ))
        .map_err(|e| e.to_string())
}
async fn deno_async_set_sys_module_inner(
    wrapper: Arc<Wrapper>,
    module_name: String,
    module_code: Option<String>,
    evaluate: bool,
) -> Result<Option<usize>, String> {
    let unwrapped: &mut DenoRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    // dont need to defined functions => ops defined in deno_extensions
    // define module by code
    if let Some(module_code) = module_code {
        let res = unwrapped
            .execute_sys_module(module_name.clone(), module_code, evaluate)
            .await?;
        return Ok(Some(res));
    }
    Ok(None)
}

pub fn deno_async_sys_to_js_binary(wrapper: &Arc<Wrapper>, data: Vec<u8>) -> Result<(), String> {
    let unwrapped: &mut DenoRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    let tokio = unwrapped.tokio.clone();
    let res = tokio.block_on(deno_async_sys_to_js_binary_inner(
        wrapper.clone(),
        unwrapped,
        data,
    ));
    res
}
async fn deno_async_sys_to_js_binary_inner(
    wrapper: Arc<Wrapper>,
    deno_runtime: &mut DenoRuntime,
    data: Vec<u8>,
) -> Result<(), String> {
    // call callback
    deno_runtime.spawn_in_scope(move |scope| {
        let deno_runtime: Result<&mut DenoRuntime, &str> =
            wrapper.as_mut().ok_or("wrapper_dropped");
        if let Ok(deno_runtime) = deno_runtime {
            let state = deno_runtime.runtime.op_state();
            let state = state.try_borrow_mut().map_err(|e| e.to_string());
            if let Ok(mut state) = state {
                let callbacks = deno_get_extension_callback(&mut state).ok_or("callback_not_init");
                if let Ok(callbacks) = callbacks {
                    // prepare args
                    let args = json_args_custom!("binary", vec!(data));
                    let args = deno_decode_args(args, scope);
                    if let Ok(args) = args {
                        //call
                        for i in callbacks.callbacks.iter() {
                            // global scope
                            let obj: deno_core::v8::Local<deno_core::v8::Value> =
                                deno_core::v8::undefined(scope).into();
                            i.open(scope).call(scope, obj, &args);
                        }
                    }
                }
            }
        }
    });
    Ok(())
}
pub fn deno_async_sys_to_js_json(wrapper: &Arc<Wrapper>, data: String) -> Result<(), String> {
    let unwrapped: &mut DenoRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    let tokio = unwrapped.tokio.clone();
    let res = tokio.block_on(deno_async_sys_to_js_json_inner(
        wrapper.clone(),
        unwrapped,
        data,
    ));
    res
}
async fn deno_async_sys_to_js_json_inner(
    wrapper: Arc<Wrapper>,
    deno_runtime: &mut DenoRuntime,
    data: String,
) -> Result<(), String> {
    // call callback
    deno_runtime.spawn_in_scope(move |scope| {
        let deno_runtime: Result<&mut DenoRuntime, &str> =
            wrapper.as_mut().ok_or("wrapper_dropped");
        if let Ok(deno_runtime) = deno_runtime {
            let state = deno_runtime.runtime.op_state();
            let state = state.try_borrow_mut().map_err(|e| e.to_string());
            if let Ok(mut state) = state {
                let callbacks = deno_get_extension_callback(&mut state).ok_or("callback_not_init");
                if let Ok(callbacks) = callbacks {
                    // prepare args
                    let arg = deno_core::serde_json::to_value(data).map_err(|e| e.to_string());
                    if let Ok(arg) = arg {
                        let args = json_args_custom!("json", arg);
                        let args = deno_decode_args(args, scope);
                        if let Ok(args) = args {
                            //call
                            for i in callbacks.callbacks.iter() {
                                // global scope
                                let obj: deno_core::v8::Local<deno_core::v8::Value> =
                                    deno_core::v8::undefined(scope).into();
                                i.open(scope).call(scope, obj, &args);
                            }
                        }
                    }
                }
            }
        }
    });
    Ok(())
}

pub fn deno_sync_eval_code(wrapper: &Arc<Wrapper>, code: String) -> Result<String, String> {
    let unwrapped: &mut DenoRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    return unwrapped.eval_to_json(code);
}
pub fn deno_async_call_function(
    wrapper: &Arc<Wrapper>,
    module: Option<(usize, String)>,
    function: String,
) -> Result<String, String> {
    let res = deno_tokio_current_thread()?
        .block_on(deno_async_call_function_inner(wrapper, module, function));
    res
}
async fn deno_async_call_function_inner(
    wrapper: &Arc<Wrapper>,
    module: Option<(usize, String)>,
    function: String,
) -> Result<String, String> {
    let unwrapped: &mut DenoRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    let args = json_args_custom!();
    if let Some(module) = module {
        return unwrapped
            .call_module_function_to_json(module, function, args)
            .await;
    } else {
        return unwrapped.call_global_function_to_json(function, args).await;
    }
}

pub fn deno_async_add_module_code(
    wrapper: &Arc<Wrapper>,
    module_name: String,
    module_code: String,
    is_main: bool,
    evaluate: bool,
) -> Result<usize, String> {
    let unwrapped: &DenoRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    let tokio = unwrapped.tokio.clone();
    let unwrapped: &mut DenoRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    let res = tokio.block_on(deno_async_add_module_code_inner(
        unwrapped,
        module_name,
        module_code,
        is_main,
        evaluate,
    ));
    res
}
async fn deno_async_add_module_code_inner(
    runtime: &mut DenoRuntime,
    module_name: String,
    module_code: String,
    is_main: bool,
    evaluate: bool,
) -> Result<usize, String> {
    if evaluate {
        runtime
            .execute_module(module_name, module_code, is_main)
            .await
    } else {
        runtime
            .preload_module(module_name, module_code, is_main)
            .await
    }
}

pub fn deno_async_eval_module(wrapper: &Arc<Wrapper>, module_id: usize) -> Result<(), String> {
    let unwrapped: &DenoRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    let tokio = unwrapped.tokio.clone();
    let unwrapped: &mut DenoRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    let res = tokio.block_on(deno_async_eval_module_inner(unwrapped, module_id));
    res
}
async fn deno_async_eval_module_inner(
    runtime: &mut DenoRuntime,
    module_id: usize,
) -> Result<(), String> {
    runtime.evaluate_module(module_id).await
}

pub fn deno_async_snapshot(module_name: String, module_code: String) -> Result<Vec<u8>, String> {
    let res =
        deno_tokio_current_thread()?.block_on(deno_async_snapshot_inner(module_name, module_code));
    res
}

async fn deno_async_snapshot_inner(
    module_name: String,
    module_code: String,
) -> Result<Vec<u8>, String> {
    // create runtime
    let loader_inner = Arc::new(std::sync::Mutex::new(DenoLoaderState::new()));
    let mut runtime = deno_core::JsRuntimeForSnapshot::try_new(deno_core::RuntimeOptions {
        startup_snapshot: None,
        extensions: vec![deno_extension::custom_extension::init_ops()],
        module_loader: Some(std::rc::Rc::new(DenoLoader::new(loader_inner.clone()))),
        ..Default::default()
    })
    .map_err(|e| e.to_string())?;
    // add module
    let specifier = DenoLoaderState::to_specifier(&module_name)
        .map_err(|_| format!("invalid_mod_name {}", module_name))?;
    let modid = runtime
        .load_side_es_module_from_code(&specifier, module_code.clone())
        .map_err(|e| e.to_string())
        .await?;
    // evaluate mod
    let result = runtime.mod_evaluate(modid);
    runtime
        .run_event_loop(deno_core::PollEventLoopOptions::default())
        .await
        .map_err(|e| e.to_string())?;
    result.await.map_err(|e| e.to_string())?;
    // generate snapshot
    let snapshot = runtime.snapshot();
    let vec: Vec<u8> = Vec::from(snapshot);
    Ok(vec)
}
