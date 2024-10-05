use super::quickjs_commons::{
    js_error_from_string, js_error_to_string, js_promise_to_json_string, js_result_to_json_string, js_value_to_data, quickjs_tokio_current_thread, CustomData, CustomMemoryUsage
};
pub use super::wrapper::Wrapper;
use crate::frb_generated::StreamSink;
use flutter_rust_bridge::frb;
use rquickjs::context::EvalOptions;
use rquickjs::function::Func;
pub use rquickjs::{
    async_with, context::intrinsic, function::Args, promise, AsyncContext, Context, Function,
    Module, Value,
};
pub use rquickjs::{qjs::JSMemoryUsage, AsyncRuntime};
use rquickjs::{CatchResultExt, IntoJs, Result as JSResult};
use std::borrow::Borrow;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::Relaxed;
pub use std::sync::Arc;
use tokio_util::sync::CancellationToken;

#[frb(external)]
#[frb(opaque)]
pub struct CustomAsyncRuntime {
    runtime: AsyncRuntime,
    context: AsyncContext,
    events: Option<Arc<StreamSink<CustomData>>>,
    cancel_token: Arc<CancellationToken>,
    count_interrupt_call: Arc<AtomicU64>,
}
pub async fn quickjs_async_create(
    memory_limit: Option<usize>,
    max_stack_size: Option<usize>,
    enable_interrupt: Option<bool>,
) -> Result<Arc<Wrapper>, String> {
    let res = tokio::task::spawn_blocking(move || {
        let rt = quickjs_tokio_current_thread()?;
        rt.block_on(async {
            let res = quickjs_async_create_inner(memory_limit, max_stack_size, enable_interrupt).await;
            res
        })
    }).await.map_err(|e|e.to_string())?;
    res
}
async fn quickjs_async_create_inner(
    memory_limit: Option<usize>,
    max_stack_size: Option<usize>,
    enable_interrupt: Option<bool>,

) -> Result<Arc<Wrapper>, String> {
    let runtime = AsyncRuntime::new().map_err(|e| e.to_string())?;
    match memory_limit {
        Some(mem) => runtime.set_memory_limit(mem).await,
        None => {}
    };
    // node default stack size is 492 kBytes (32-bit) and 984 kBytes (64-bit).
    match max_stack_size {
        Some(max) => runtime.set_max_stack_size(max).await,
        None => {}
    }
    // set cancel token
    let enable_interrupt = enable_interrupt.unwrap_or(true);
    let count_interrupt_call = Arc::new(AtomicU64::new(0));
    let cancel_token = Arc::new(CancellationToken::new());
    if enable_interrupt { 
        let count_interrupt_call_clone = count_interrupt_call.clone();
        let cancel_token_clone = cancel_token.clone();
        let callback: Box<dyn FnMut() -> bool + Send + 'static> = Box::new(move || -> bool {
            let _ = count_interrupt_call_clone.fetch_update(Relaxed, Relaxed, |x| {
                if x == u64::MAX {
                    None
                } else {
                    Some(x + 1u64)
                }
            });
            return cancel_token_clone.is_cancelled();
        });
        runtime.set_interrupt_handler(Some(callback)).await;
    }
    //
    let res_ctx = AsyncContext::full(&runtime)
        .await
        .map_err(|e| e.to_string());
    let context = res_ctx?;
    let _ = context
        .with(|ctx| {
            unsafe {
                let ptr = ctx.as_raw().as_ptr();
                rquickjs::qjs::JS_AddIntrinsicBigInt(ptr);
                rquickjs::qjs::JS_AddIntrinsicBigFloat(ptr);
                rquickjs::qjs::JS_AddIntrinsicBigDecimal(ptr);
                rquickjs::qjs::JS_AddIntrinsicProxy(ptr);
                //rquickjs::qjs::JS_AddIntrinsicTypedArrays(ptr);
            }
        })
        .await;
    let res = CustomAsyncRuntime {
        context,
        runtime,
        events: None,
        cancel_token,
        count_interrupt_call,
    };
    let res_safe = Arc::new(Wrapper::new(res));
    Ok(res_safe)
}

pub async fn quickjs_async_interrupt(wrapper: &Arc<Wrapper>) -> Result<bool, String> {
    let wrapper = wrapper.clone();
    let res = tokio::task::spawn_blocking(|| {
        let rt = quickjs_tokio_current_thread()?;
        rt.block_on(async {
            let res = quickjs_async_interrupt_inner(wrapper).await;
            res
        })
    }).await.map_err(|e|e.to_string())?;
    res
}
async fn quickjs_async_interrupt_inner(wrapper: Arc<Wrapper>) -> Result<bool, String> {
    let unwrapped: &mut CustomAsyncRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    unwrapped.cancel_token.cancel();
    Ok(unwrapped.cancel_token.is_cancelled())
}

pub async fn quickjs_async_dispose(wrapper: &Arc<Wrapper>) -> Result<(), String> {
    let wrapper = wrapper.clone();
    let res = tokio::task::spawn_blocking(|| {
        let rt = quickjs_tokio_current_thread()?;
        rt.block_on(async {
            let res = quickjs_async_dispose_inner(wrapper).await;
            res
        })
    }).await.map_err(|e|e.to_string())?;
    res
}
async fn quickjs_async_dispose_inner(wrapper: Arc<Wrapper>) -> Result<(), String> {
    let unwrapped: &mut CustomAsyncRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    let _: Result<(), String> = async_with!(unwrapped.context.borrow() => |ctx| {
        ctx.run_gc();
        ctx.execute_pending_job();
        Ok(())
    })
    .await;
    unwrapped.runtime.run_gc().await;
    unwrapped
        .runtime
        .execute_pending_job()
        .await
        .map_err(|e| e.to_string())?;
    drop(wrapper.to_owned());
    Ok(())
}

pub async fn quickjs_async_get_memory_usage(wrapper: &Arc<Wrapper>) -> Result<CustomMemoryUsage, String> {
    let wrapper = wrapper.clone();
    let res = tokio::task::spawn_blocking(|| {
        let rt = quickjs_tokio_current_thread()?;
        rt.block_on(async {
            let res = quickjs_async_get_memory_usage_inner(wrapper).await;
            res
        })
    }).await.map_err(|e|e.to_string())?;
    res
}
async fn quickjs_async_get_memory_usage_inner(
    wrapper: Arc<Wrapper>,
) -> Result<CustomMemoryUsage, String> {
    let unwrapped: &mut CustomAsyncRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    let res: rquickjs::qjs::JSMemoryUsage = unwrapped.runtime.memory_usage().await;
    Ok(CustomMemoryUsage {
        malloc_size: res.malloc_size,
        memory_used_size: res.memory_used_size,
        count_interrupt_calls: unwrapped.count_interrupt_call.load(Relaxed),
    })
}

pub fn quickjs_async_listen(
    wrapper: &Arc<Wrapper>,
    events: StreamSink<CustomData>,
) -> Result<(), String> {
    let unwrapped: &mut CustomAsyncRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    unwrapped.events = Some(Arc::new(events));
    Ok(())
}

pub async fn quickjs_async_set_sys_module(
    wrapper: &Arc<Wrapper>,
    module_name: String,
    module_code: Option<String>,
    module_bytecode: Option<Vec<u8>>,
    js_to_sys_name: String,
) -> Result<(), String> {
    let wrapper = wrapper.clone();
    let res = tokio::task::spawn_blocking(|| {
        let rt = quickjs_tokio_current_thread()?;
        rt.block_on(async {
            let res = quickjs_async_set_sys_module_inner(wrapper, module_name, module_code, module_bytecode, js_to_sys_name).await;
            res
        })
    }).await.map_err(|e|e.to_string())?;
    res
}
async fn quickjs_async_set_sys_module_inner(
    wrapper: Arc<Wrapper>,
    module_name: String,
    module_code: Option<String>,
    module_bytecode: Option<Vec<u8>>,
    js_to_sys_name: String,
) -> Result<(), String> {
    let unwrapped: &mut CustomAsyncRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    let events = unwrapped
        .events
        .as_ref()
        .ok_or("stream_has_not_been_init")?;
    let events_clone = events.clone();
    // define a function to send data
    let res: Result<(), String> = async_with!(unwrapped.context.borrow() => |ctx|{
        let ctx_clone = ctx.clone();
        let ctx_clone2 = ctx.clone();
        let ctx_clone3 = ctx.clone();
        // define a function to send data (before module)
        let js_to_sys_function = Func::from(move |val: Value| -> JSResult<bool> {
            // covert val to binary
            let json_or_binary = js_value_to_data(val)?;
            // result promise
            events_clone.add(json_or_binary).map_err(|e| js_error_from_string(e.to_string()))?;
            return Ok(true);
        });
        ctx.globals().set(js_to_sys_name, js_to_sys_function).map_err(|e| e.to_string())?;
        
        // load module by byte
        if let Some(module_bytecode_safe) = module_bytecode {
            let bytes: &[u8] = &module_bytecode_safe;
            let module = unsafe { Module::load(ctx_clone3.clone(), bytes).catch(ctx_clone3.borrow()).map_err(|e| e.to_string())? };
            // check name
            let module_loaded_name = module.name::<String>().map_err(|e|e.to_string())?;
            if module_loaded_name != module_name {
                return Err(format!("bad_module_name '{}' != '{}'", module_name, module_loaded_name));
            }
            // import then eval (to keep module in resolved struct)
            let promise = Module::import(ctx.borrow(), module_name).catch(ctx.borrow()).map_err(|e| e.to_string())?;
            let __ = promise.finish::<rquickjs::Object>().map_err(|e| js_error_to_string(e, ctx_clone3))?;
        } // define module by code
        else if let Some(module_code_safe) = module_code {
            let _ = Module::declare(ctx_clone.clone(),module_name.clone(),module_code_safe).catch(ctx_clone.borrow()).map_err(|e| e.to_string())?;
            let promise = Module::import(ctx.borrow(), module_name.clone()).catch(ctx.borrow()).map_err(|e| e.to_string())?;
            let __ = promise.finish::<rquickjs::Object>().map_err(|e| js_error_to_string(e, ctx_clone2))?;
        }
        // module and function has been defined
        Ok(())
    })
    .await;
    res
}

pub async fn quickjs_async_sys_to_js_binary(
    wrapper: &Arc<Wrapper>,
    module_name: String,
    trigger_name: String,
    data: Vec<u8>,
) -> Result<i32, String> {
    let wrapper = wrapper.clone();
    let res = tokio::task::spawn_blocking(|| {
        let rt = quickjs_tokio_current_thread()?;
        rt.block_on(async {
            let res = quickjs_async_sys_to_js_binary_inner(
                wrapper,
                module_name,
                trigger_name,
                data,
            ).await;
            res
        })
    }).await.map_err(|e|e.to_string())?;
    res
}
async fn quickjs_async_sys_to_js_binary_inner(
    wrapper: Arc<Wrapper>,
    module_name: String,
    trigger_name: String,
    data: Vec<u8>,
) -> Result<i32, String> {
    let unwrapped: &mut CustomAsyncRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    let res: Result<i32, String> = async_with!(unwrapped.context.borrow() => |ctx|{
        let ctx_clone = ctx.clone();
        let ctx_clone2 = ctx.clone();
        let ctx_clone3 = ctx.clone();
        let promise = Module::import(ctx.borrow(), module_name).catch(ctx.borrow()).map_err(|e| e.to_string())?;
        let module = promise
            .finish::<rquickjs::Object>()
            .map_err(|e| js_error_to_string(e, ctx_clone3))?;
        let res = module
            .get::<&str, Function>(trigger_name.as_str()).catch(ctx.borrow())
            .map_err(|e| e.to_string())?;
        let mut args = Args::new(ctx, 1);
        args.push_arg(data.into_js(ctx_clone.borrow())).map_err(|e| e.to_string())?;
        let resi = res.call_arg::<rquickjs::Value>(args).catch(ctx_clone2.borrow())
            .map_err(|e| e.to_string())?;
        match resi.as_int() {
            Some(res_safe) => Ok(res_safe),
            None => Ok(0),
        }
    })
    .await;
    res
}

pub async fn quickjs_async_sys_to_js_json(
    wrapper: &Arc<Wrapper>,
    module_name: String,
    trigger_name: String,
    data: String,
) -> Result<i32, String> {
    let wrapper = wrapper.clone();
    let res = tokio::task::spawn_blocking(|| {
        let rt = quickjs_tokio_current_thread()?;
        rt.block_on(async {
            let res = quickjs_async_sys_to_js_json_inner(wrapper, module_name, trigger_name, data).await;
            res
        })
    }).await.map_err(|e|e.to_string())?;
    res
}
async fn quickjs_async_sys_to_js_json_inner(
    wrapper: Arc<Wrapper>,
    module_name: String,
    trigger_name: String,
    data: String,
) -> Result<i32, String> {
    let unwrapped: &mut CustomAsyncRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    let res: Result<i32, String> = async_with!(unwrapped.context.borrow() => |ctx|{
        let ctx_clone = ctx.clone();
        let ctx_clone2 = ctx.clone();
        let ctx_clone3 = ctx.clone();
        let promise = Module::import(ctx.borrow(), module_name).catch(ctx.borrow()).map_err(|e| e.to_string())?;
        let module = promise
            .finish::<rquickjs::Object>()
            .map_err(|e| js_error_to_string(e, ctx))?;
        let res = module
            .get::<&str, Function>(trigger_name.as_str()).catch(ctx_clone3.borrow())
            .map_err(|e| e.to_string())?;
        let mut args = Args::new(ctx_clone3, 1);
        args.push_arg(data.into_js(ctx_clone.borrow())).map_err(|e| e.to_string())?;
        let resi = res.call_arg::<rquickjs::Value>(args).catch(ctx_clone2.borrow())
            .map_err(|e| e.to_string())?;
        match resi.as_int() {
            Some(res_safe) => Ok(res_safe),
            None => Ok(0),
        }
    })
    .await;
    res
}

pub async fn quickjs_async_eval_code(
    wrapper: &Arc<Wrapper>,
    code: String,
    backtrace_barrier: bool,
    global: bool,
    promise: bool,
    strict: bool,
) -> Result<String, String> {
    let wrapper = wrapper.clone();
    let res = tokio::task::spawn_blocking(move || {
        let rt = quickjs_tokio_current_thread()?;
        rt.block_on(async {
            let res = quickjs_async_eval_code_inner(wrapper, code, backtrace_barrier, global, promise, strict).await;
            res
        })
    }).await.map_err(|e|e.to_string())?;
    res
}
async fn quickjs_async_eval_code_inner(
    wrapper: Arc<Wrapper>,
    code: String,
    backtrace_barrier: bool,
    global: bool,
    promise: bool,
    strict: bool,
) -> Result<String, String> {
    let unwrapped: &mut CustomAsyncRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    let token = unwrapped.cancel_token.clone();
    let res: Result<String, String> = async_with!(unwrapped.context.borrow() => |ctx|{
        let mut opts = EvalOptions::default();
        opts.backtrace_barrier = backtrace_barrier;
        opts.promise = promise;
        opts.strict = strict;
        opts.global = global;
        let result = ctx
            .eval_with_options::<Value, String>(code, opts)
            .catch(ctx.borrow())
            .map_err(|e| e.to_string());
        match result {
            Ok(result_safe) => {
                tokio::select! {
                    val= js_result_to_json_string(result_safe, ctx)=>{
                        let res = val?;
                        return Ok(res);
                    },
                    _ = token.cancelled()=> {
                        return Err("cancelled".to_owned());
                    }
                };
            },
            Err(err)=>Err(err),
        }
    })
    .await;
    res
}

pub async fn quickjs_async_add_module_code(
    wrapper: &Arc<Wrapper>,
    module_name: String,
    module_code: String,
) -> Result<(), String> {
    let wrapper = wrapper.clone();
    let res = tokio::task::spawn_blocking(move || {
        let rt = quickjs_tokio_current_thread()?;
        rt.block_on(async {
            let res = quickjs_async_add_module_code_inner(wrapper, module_name, module_code).await;
            res
        })
    }).await.map_err(|e|e.to_string())?;
    res
}
async fn quickjs_async_add_module_code_inner(
    wrapper: Arc<Wrapper>,
    module_name: String,
    module_code: String,
) -> Result<(), String> {
    let unwrapped: &mut CustomAsyncRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    let res: Result<(), String> = async_with!(unwrapped.context.borrow() => |ctx|{
        let _ = Module::declare(ctx.clone(), module_name, module_code).catch(ctx.borrow())
            .map_err(|e| e.to_string())?;
        // should not eval module (it will be imported)
        //let (_, promise) = module.eval().map_err(|e| e.to_string())?;
        //promise.finish::<rquickjs::Value>().map_err(|e| e.to_string())?;
        Ok(())
    })
    .await;
    res
}

pub async fn quickjs_async_add_module_bytecode(
    wrapper: &Arc<Wrapper>,
    module_name: String,
    module_bytecode: Vec<u8>,
) -> Result<(), String> {
    let wrapper = wrapper.clone();
    let res = tokio::task::spawn_blocking(move || {
        let rt = quickjs_tokio_current_thread()?;
        rt.block_on(async {
            let res = quickjs_async_add_module_bytecode_inner(wrapper, module_name, module_bytecode).await;
            res
        })
    }).await.map_err(|e|e.to_string())?;
    res
}
async fn quickjs_async_add_module_bytecode_inner(
    wrapper: Arc<Wrapper>,
    module_name: String,
    module_bytecode: Vec<u8>,
) -> Result<(), String> {
    let unwrapped: &mut CustomAsyncRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    let res: Result<(), String> = async_with!(unwrapped.context.borrow() => |ctx|{
        let bytes: &[u8] = &module_bytecode;
        let module =
            unsafe { Module::load(ctx.clone(), bytes).catch(ctx.borrow()).map_err(|e| e.to_string())? };
        let module_loaded_name = module.name::<String>().map_err(|e|e.to_string())?;
        if module_loaded_name != module_name {
            return Err(format!("bad_module_name '{}' != '{}'", module_name, module_loaded_name));
        }
        // should not eval => will be evaluated on import
        //let (_, promise) = module.eval().map_err(|e| e.to_string())?;
        //promise.finish::<rquickjs::Value>().map_err(|e| e.to_string())?;
        Ok(())
    })
    .await;
    res
}

pub async fn quickjs_async_set_global_value(
    wrapper: &Arc<Wrapper>,
    name: String,
    value: String,
) -> Result<(), String> {
    let wrapper = wrapper.clone();
    let res = tokio::task::spawn_blocking(move || {
        let rt = quickjs_tokio_current_thread()?;
        rt.block_on(async {
            let res = quickjs_async_set_global_value_inner(wrapper, name, value).await;
            res
        })
    }).await.map_err(|e|e.to_string())?;
    res
}
async fn quickjs_async_set_global_value_inner(
    wrapper: Arc<Wrapper>,
    name: String,
    value: String,
) -> Result<(), String> {
    let unwrapped: &mut CustomAsyncRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    let res: Result<(), String> = async_with!(unwrapped.context.borrow() => |ctx|{
        let val = ctx.json_parse_ext(value, true).map_err(|e| e.to_string())?;
        ctx.globals().set(name, val).map_err(|e| e.to_string())?;
        Ok(())
    })
    .await;
    res
}
pub async fn quickjs_async_execute_pending(wrapper: &Arc<Wrapper>) -> Result<bool, String> {
    let wrapper = wrapper.clone();
    let res = tokio::task::spawn_blocking(move || {
        let rt = quickjs_tokio_current_thread()?;
        rt.block_on(async {
            let res = quickjs_async_execute_pending_inner(wrapper).await;
            res
        })
    }).await.map_err(|e|e.to_string())?;
    res
}
async fn quickjs_async_execute_pending_inner(wrapper: Arc<Wrapper>) -> Result<bool, String> {
    let unwrapped: &mut CustomAsyncRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    let res = unwrapped
        .runtime
        .execute_pending_job()
        .await
        .map_err(|e| e.to_string())?;
    Ok(res)
}
pub async fn quickjs_async_execute_idle(wrapper: &Arc<Wrapper>) -> Result<(), String> {
    let wrapper = wrapper.clone();
    let res = tokio::task::spawn_blocking(move || {
        let rt = quickjs_tokio_current_thread()?;
        rt.block_on(async {
            let res = quickjs_async_execute_idle_inner(wrapper).await;
            res
        })
    }).await.map_err(|e|e.to_string())?;
    res
}
async fn quickjs_async_execute_idle_inner(wrapper: Arc<Wrapper>) -> Result<(), String> {
    let unwrapped: &mut CustomAsyncRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    let res = unwrapped.runtime.idle().await;
    Ok(res)
}

pub async fn quickjs_async_is_pending(wrapper: &Arc<Wrapper>) -> Result<bool, String> {
    let wrapper = wrapper.clone();
    let res = tokio::task::spawn_blocking(move || {
        let rt = quickjs_tokio_current_thread()?;
        rt.block_on(async {
            let res = quickjs_async_is_pending_inner(wrapper).await;
            res
        })
    }).await.map_err(|e|e.to_string())?;
    res
}
async fn quickjs_async_is_pending_inner(wrapper: Arc<Wrapper>) -> Result<bool, String> {
    let unwrapped: &mut CustomAsyncRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    let res = unwrapped.runtime.is_job_pending().await;
    Ok(res)
}
pub async fn quickjs_async_compile(
    wrapper: &Arc<Wrapper>,
    module_name: String,
    module_code: String,
) -> Result<Vec<u8>, String> {
    let wrapper = wrapper.clone();
    let res = tokio::task::spawn_blocking(move || {
        let rt = quickjs_tokio_current_thread()?;
        rt.block_on(async {
            let res = quickjs_async_compile_inner(wrapper, module_name, module_code).await;
            res
        })
    }).await.map_err(|e|e.to_string())?;
    res
}
async fn quickjs_async_compile_inner(
    wrapper: Arc<Wrapper>,
    module_name: String,
    module_code: String,
) -> Result<Vec<u8>, String> {
    let unwrapped: &mut CustomAsyncRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    // define a function to send data
    let res: Result<Vec<u8>, String> = async_with!(unwrapped.context.borrow() => |ctx|{
        let ctx_clone = ctx.clone();
        let module = Module::declare(ctx_clone.clone(),module_name,module_code).catch(ctx_clone.borrow()).map_err(|e| e.to_string())?;
        module.write(false).map_err(|e|e.to_string())
    })
    .await;
    res
}
pub async fn quickjs_async_eval_module(
    wrapper: &Arc<Wrapper>,
    module_bytes: Vec<u8>,
    module_name: String,
) -> Result<String, String> {
    let wrapper = wrapper.clone();
    let res = tokio::task::spawn_blocking(move || {
        let rt = quickjs_tokio_current_thread()?;
        rt.block_on(async {
            let res = quickjs_async_eval_module_inner(wrapper, module_bytes, module_name).await;
            res
        })
    }).await.map_err(|e|e.to_string())?;
    res
}
async fn quickjs_async_eval_module_inner(
    wrapper: Arc<Wrapper>,
    module_bytes: Vec<u8>,
    module_name: String,
) -> Result<String, String> {
    let unwrapped: &mut CustomAsyncRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    let token = unwrapped.cancel_token.clone();
    // define a function to send data
    let res: Result<String, String> = async_with!(unwrapped.context.borrow() => |ctx|{
        let ctx_clone = ctx.clone();
        let ctx_clone2 = ctx.clone();
        let bytes = module_bytes.borrow();
        let module = unsafe{
            let module = Module::load(ctx_clone.clone(), bytes).catch(ctx_clone.borrow()).map_err(|e| e.to_string())?;
            module
        };
        let module_loaded_name = module.name::<String>().map_err(|e|e.to_string())?;
        if module_loaded_name != module_name {
            return Err(format!("bad_module_name '{}' != '{}'", module_name, module_loaded_name));
        }
        let (_, promise) = module.eval().catch(ctx_clone2.borrow()).map_err(|e| e.to_string())?;
        tokio::select! {
            val= js_promise_to_json_string(promise, ctx)=>{
                let res = val?;
                return Ok(res);
            },
            _ = token.cancelled()=> {
                return Err("cancelled".to_owned());
            }
        };
    })
    .await;
    res
}