use flutter_rust_bridge::frb;
use rquickjs::qjs::{JS_FreeCString, JS_GetProperty, JS_ToCString};
pub use rquickjs::{
    async_with, context::intrinsic, function::Args, promise, AsyncContext, Context, Function,
    Module, Value,
};
pub use rquickjs::{qjs::JSMemoryUsage, AsyncRuntime};
use rquickjs::{Ctx, Promise};
use std::borrow::Borrow;
use std::ffi::CStr;
pub use std::sync::Arc;

#[frb(external)]
#[frb(non_opaque)]
#[derive(Clone, Debug)]
pub struct CustomData {
    pub binary: Option<Vec<u8>>,
    pub json: Option<String>,
}

#[frb(external)]
#[frb(non_opaque)]
pub struct CustomMemoryUsage {
    pub malloc_size: i64,
    pub memory_used_size: i64,
    pub count_interrupt_calls: u64,
}

pub(crate) fn js_error_from_string(error: String) -> rquickjs::Error {
    return rquickjs::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, error));
}

pub(crate) fn js_value_to_data<'js>(value: Value<'js>) -> Result<CustomData, rquickjs::Error> {
    let binary_opt = js_value_to_uint8array(value.borrow()).map(|safe_binary| CustomData {
        binary: Some(safe_binary),
        json: None,
    });
    let json_or_binary_opt = binary_opt.map_or_else(
        |_| {
            // convert value to json
            return js_value_to_string(value).map(|safe_json| CustomData {
                binary: None,
                json: Some(safe_json),
            });
        },
        |safe_binary| Ok(safe_binary),
    );
    let json_or_binary = json_or_binary_opt.map_err(|e| js_error_from_string(e));
    return json_or_binary;
}

fn js_value_to_uint8array(val: &Value) -> Result<Vec<u8>, String> {
    if val.is_object() {
        let object = val.as_object();
        match object {
            Some(safe_object) => {
                if safe_object.is_typed_array::<u8>() {
                    let typearray = safe_object.as_typed_array::<u8>();
                    match typearray {
                        Some(safe_typearray) => {
                            // send bytes
                            let len = safe_typearray.len();
                            let mut binary: Vec<u8> = Vec::with_capacity(len);
                            // copy data
                            for i in 0..len {
                                let byte = safe_typearray.get(i.to_string());
                                match byte {
                                    Ok(safe_byte) => {
                                        binary.push(safe_byte);
                                    }
                                    Err(_) => {
                                        return Err("failed_to_parse_uint8array".to_owned());
                                    }
                                }
                            }
                            return Ok(binary);
                        }
                        None => {}
                    }
                }
            }
            None => {}
        }
    }
    Err("could_not_convert_to_uint8array".to_owned())
}

fn js_value_to_string(val: Value) -> Result<String, String> {
    let str: Result<rquickjs::String, Value> = val.try_into_string();
    match str {
        Ok(str_safe) => {
            let rust_str = str_safe.to_string();
            match rust_str {
                Ok(safe) => {
                    return Ok(safe);
                }
                Err(_) => {}
            }
        }
        Err(_) => {}
    }
    Err("could_not_convert_to_json".to_owned())
}
pub(crate) async fn js_result_to_json_string<'js>(
    value: Value<'js>,
    ctx_safe: Ctx<'js>,
) -> Result<String, String> {
    if value.is_promise() {
        let ctx_clone = ctx_safe.clone();
        // wait promise and stringify
        let promise = value
            .try_into_promise()
            .map_err(|_| "coult_not_parse_promise")?;
        let json = js_promise_to_json_string(promise, ctx_clone).await;
        return json;
    } else {
        // stringify value
        return js_object_to_json_string(value, ctx_safe);
    }
}

pub(crate) async fn js_promise_to_json_string<'js>(
    promise: Promise<'js>,
    ctx_safe: Ctx<'js>,
) -> Result<String, String> {
    let object = promise
        .into_future::<Value>()
        .await
        .map_err(|e| js_error_to_string(e, ctx_safe.clone()))?;
    let result = js_object_to_json_string(object, ctx_safe);
    return result;
}

pub(crate) fn js_object_to_json_string<'js>(
    value: Value<'js>,
    ctx_safe: Ctx<'js>,
) -> Result<String, String> {
    // stringify value
    let val: Option<rquickjs::String> = ctx_safe
        .json_stringify(value.borrow())
        .map_err(|e| e.to_string())?;
    match val {
        // undefined
        None => Ok("null".to_owned()),
        // quickjs string to rust string
        Some(safe_val) => safe_val.to_string().map_err(|e| e.to_string()),
    }
}
pub(crate) fn js_error_to_string<'js>(error: rquickjs::Error, ctx_safe: Ctx<'js>) -> String {
    let catched = ctx_safe.catch();
    let mut final_error = error.to_string();
    if catched.is_error() && catched.is_object() {
        unsafe {
            let ptr = ctx_safe.as_raw().as_ptr();
            // get description
            let error_str = JS_ToCString(ptr, catched.as_raw());
            let error_cstr = CStr::from_ptr(error_str);
            let error_rust_str = error_cstr.to_str();
            match error_rust_str {
                Ok(error_safe) => {
                    final_error = error_safe.to_owned();
                }
                Err(_) => {}
            }
            JS_FreeCString(ptr, error_str);
            // get stack
            let stack_js_value =
                JS_GetProperty(ptr, catched.as_raw(), rquickjs::qjs::JS_ATOM_stack);
            let stack_value = Value::from_raw(ctx_safe, stack_js_value);
            let stack_value_string = stack_value.try_into_string();
            match stack_value_string {
                Ok(stack_value_safe) => {
                    let stack_rust_string = js_string_to_rust_string(&stack_value_safe)
                        .unwrap_or_else(|_| "".to_owned());
                    final_error = format!("{} \n {}", final_error, stack_rust_string);
                }
                Err(_) => {}
            }
            // dont need => see trait Drop on Value
            //JS_FreeValue(ptr, stack_js_value);
        }
    }
    return final_error;
}

pub(crate) fn js_string_to_rust_string<'js>(str: &rquickjs::String<'js>) -> Result<String, String> {
    let rust_string = str.to_string();
    let str_safe = rust_string.map_err(|e| e.to_string())?;
    return Ok(str_safe);
}

#[allow(dead_code)]
pub(crate) fn quickjs_tokio_current_thread() -> Result<tokio::runtime::Runtime, String> {
    return tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string());
}
#[allow(dead_code)]
pub(crate) fn quickjs_tokio_multi_thread() -> Result<tokio::runtime::Runtime, String> {
    return tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string());
}
#[allow(dead_code)]
pub(crate) async fn quickjs_spawn_task<F, T>(single_thread_future: F) -> Result<T, String>
where
    F: std::future::Future<Output = Result<T, String>> + Send + 'static,
    T: Send + 'static,
{
    let res = tokio::task::spawn_blocking(move || {
        let rt = quickjs_tokio_current_thread()?;
        rt.block_on(single_thread_future)
    })
    .await
    .map_err(|e| e.to_string())?;
    res
}
