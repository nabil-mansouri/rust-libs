use flutter_rust_bridge::{frb, DartFnFuture};
use rquickjs::context::EvalOptions;
use rquickjs::function::Func;
pub use rquickjs::{
    async_with, context::intrinsic, function::Args, promise, AsyncContext, Context, Function,
    Module, Value,
};
pub use rquickjs::{qjs::JSMemoryUsage, AsyncRuntime};
use rquickjs::{CatchResultExt, IntoJs, Runtime};
use std::borrow::Borrow;
pub use std::sync::Arc;

use super::quickjs_commons::{
    js_result_to_json_string, js_value_to_data, CustomData,
    CustomMemoryUsage,
};
use super::wrapper::Wrapper;
//TODO implement compile using JS_WriteObject: https://github.com/bellard/quickjs/blob/master/fuzz/fuzz_compile.c#L37

#[frb(opaque)]
pub struct CustomSyncRuntime {
    runtime: Runtime,
    context: Context,
}

pub fn quickjs_sync_create(memory_limit: Option<usize>) -> Result<Arc<Wrapper>, String> {
    let runtime = Runtime::new().map_err(|e| e.to_string())?;
    match memory_limit {
        Some(mem) => runtime.set_memory_limit(mem),
        None => {}
    };
    let ctx = Context::builder()
        .with::<intrinsic::All>()
        .build(&runtime)
        .map_err(|e| e.to_string());
    let context = ctx?;
    let res = CustomSyncRuntime {
        context,
        runtime,
    };
    Ok(Arc::new(Wrapper::new(res)))
}

pub fn quickjs_sync_interrupt(wrapper: &Arc<Wrapper>) -> Result<(), String> {
    let unwrapped: &mut CustomSyncRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    let callback: Box<dyn FnMut() -> bool + Send + 'static> = Box::new(|| -> bool {
        return true;
    });
    unwrapped.runtime.set_interrupt_handler(Some(callback));
    Ok(())
}

pub fn quickjs_sync_dispose(wrapper: &Arc<Wrapper>) -> Result<(), String> {
    let unwrapped: &mut CustomSyncRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    unwrapped.context.clone().with(|ctx| {
        ctx.run_gc();
        ctx.execute_pending_job();
    });
    unwrapped.runtime.run_gc();
    unwrapped
        .runtime
        .execute_pending_job()
        .map_err(|e| e.to_string())?;
    drop(wrapper.to_owned());
    Ok(())
}

pub fn quickjs_sync_get_memory_usage(wrapper: &Arc<Wrapper>) -> Result<CustomMemoryUsage, String> {
    let unwrapped: &mut CustomSyncRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    let res: rquickjs::qjs::JSMemoryUsage = unwrapped.runtime.memory_usage();
    Ok(CustomMemoryUsage {
        malloc_size: res.malloc_size,
        memory_used_size: res.memory_used_size,
        count_interrupt_calls: 0u64
    })
}

pub fn quickjs_sync_set_sys_module(
    wrapper: &Arc<Wrapper>,
    module_name: String,
    module_code: String,
    js_to_sys_name: String,
    js_to_sys: impl Fn(CustomData) -> DartFnFuture<()> + Send + 'static,
) -> Result<(), String> {
    let unwrapped: &mut CustomSyncRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    let rt = Arc::new(
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| e.to_string())?,
    );
    let rt_clone1 = rt.clone();
    let rt_clone2 = rt.clone();
    let js_to_sys_boxed = Arc::new(js_to_sys);
    let res: Result<(), String> = unwrapped.context.clone().with(|ctx| {
        // js to sys function
        let js_to_sys_function = Func::from(  move |val: Value|  -> rquickjs::Result<bool>{
            // binary
            let data =js_value_to_data(val)?;
            rt_clone1.block_on(js_to_sys_boxed(data));
            return Ok(true);
        });
        ctx
            .globals()
            .set(js_to_sys_name, js_to_sys_function)
            .map_err(|e| e.to_string())?;
        // define module
        let module = Module::declare(ctx, module_name, module_code)
            .map_err(|e| e.to_string())?;
        let (_, promise) = module.eval().map_err(|e| e.to_string())?;
        rt_clone2
            .block_on(promise.into_future::<rquickjs::Value>())
            .map_err(|e| e.to_string())?;
        Ok(())
    });
    res.map_err(|e| e.to_string())
}

pub fn quickjs_sync_sys_to_js_binary(
    wrapper: &Arc<Wrapper>,
    module_name: String,
    trigger_name: String,
    data: Vec<u8>,
) -> Result<i32, String> {
    let unwrapped: &mut CustomSyncRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    let rt = Arc::new(
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| e.to_string())?,
    );
    let res: Result<i32, String> = unwrapped.context.clone().with(|ctx| {
        let clone = ctx.clone();
        let promise =
            Module::import(ctx.borrow(), module_name).map_err(|e| e.to_string())?;
        let module = rt
            .block_on(promise.into_future::<rquickjs::Object>())
            .map_err(|e| e.to_string())?;
        let res = module
            .get::<&str, Function>(trigger_name.as_str())
            .map_err(|e| e.to_string())?;
        let mut args = Args::new(ctx, 1);
        args.push_arg(data.into_js(clone.borrow()))
            .map_err(|e| e.to_string())?;
        let resi = res
            .call_arg::<rquickjs::Value>(args)
            .map_err(|e| e.to_string())?;
        match resi.as_int() {
            Some(res_safe) => Ok(res_safe),
            None => Ok(0),
        }
    });
    res
}

pub fn quickjs_sync_sys_to_js_json(
    wrapper: &Arc<Wrapper>,
    module_name: String,
    trigger_name: String,
    data: String,
) -> Result<i32, String> {
    let unwrapped: &mut CustomSyncRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    let rt = Arc::new(
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| e.to_string())?,
    );
    let res: Result<i32, String> = unwrapped.context.clone().with(|ctx| {
        let clone = ctx.clone();
        let promise = Module::import(ctx.borrow(), module_name).map_err(|e| e.to_string())?;
        let module = rt
            .block_on(promise.into_future::<rquickjs::Object>())
            .map_err(|e| e.to_string())?;
        let res = module
            .get::<&str, Function>(trigger_name.as_str())
            .map_err(|e| e.to_string())?;
        let mut args = Args::new(ctx, 1);
        args.push_arg(data.into_js(clone.borrow()))
            .map_err(|e| e.to_string())?;
        let resi = res
            .call_arg::<rquickjs::Value>(args)
            .map_err(|e| e.to_string())?;
        match resi.as_int() {
            Some(res_safe) => Ok(res_safe),
            None => Ok(0),
        }
    });
    res
}

pub fn quickjs_sync_eval_code(
    wrapper: &Arc<Wrapper>,
    code: String,
    backtrace_barrier: bool,
    global: bool,
    promise: bool,
    strict: bool,
) -> Result<String, String> {
    let unwrapped: &mut CustomSyncRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    let rt = Arc::new(
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| e.to_string())?,
    );
    let res = unwrapped.context.clone().with(|context| {
        let mut opts = EvalOptions::default();
        opts.backtrace_barrier = backtrace_barrier;
        opts.promise = promise;
        opts.strict = strict;
        opts.global = global;
        let result = context
            .eval_with_options::<Value, String>(code, opts)
            .catch(context.borrow())
            .map_err(|e| e.to_string());
        match result {
            Ok(result_safe) => {
                return rt.block_on(js_result_to_json_string(result_safe, context));
            }
            Err(err) => Err(err),
        }
    });
    res
}

pub fn quickjs_sync_add_module(
    wrapper: &Arc<Wrapper>,
    module_name: String,
    module_code: String,
) -> Result<(), String> {
    let unwrapped: &mut CustomSyncRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    let rt = Arc::new(
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| e.to_string())?,
    );
    let res: Result<(), String> = unwrapped.context.clone().with(|ctx| {
        let module = Module::declare(ctx, module_name, module_code)
            .map_err(|e| e.to_string())?;
        let (_, promise) = module.eval().map_err(|e| e.to_string())?;
        rt.block_on(promise.into_future::<rquickjs::Value>())
            .map_err(|e| e.to_string())?;
        Ok(())
    });
    res
}

pub fn quickjs_sync_add_module_bytes(
    wrapper: &Arc<Wrapper>,
    module_bytes: Vec<u8>,
) -> Result<(), String> {
    let unwrapped: &mut CustomSyncRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    let rt = Arc::new(
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| e.to_string())?,
    );
    let res: Result<(), String> = unwrapped.context.clone().with(|ctx| {
        let bytes: &[u8] = &module_bytes;
        let module =
            unsafe { Module::load(ctx, bytes).map_err(|e| e.to_string())? };
        let (_, promise) = module.eval().map_err(|e| e.to_string())?;
        rt.block_on(promise.into_future::<rquickjs::Value>())
            .map_err(|e| e.to_string())?;
        Ok(())
    });
    res
}

pub fn quickjs_sync_set_global_value(
    wrapper: &Arc<Wrapper>,
    name: String,
    value: String,
) -> Result<(), String> {
    let unwrapped: &mut CustomSyncRuntime = wrapper.as_mut().ok_or("wrapper_dropped")?;
    unwrapped.context.clone().with(|ctx| {
        let val = ctx.json_parse_ext(value, true).map_err(|e| e.to_string())?;
        ctx.globals().set(name, val).map_err(|e| e.to_string())?;
        Ok(())
    })
}
