use deno_core::v8::{self, Global, HandleScope};
// see rustyscript-0.8.3/src/inner_runtime.rs
pub(crate) async fn deno_call_function_to_json(
    deno_runtime: &mut deno_core::JsRuntime, 
    module: Option<(usize, String)>,
    function: String,
    args: &impl serde::ser::Serialize,
) -> Result<String, String> {
    let res = deno_call_function(deno_runtime, module, function, args).await?;
    let scope = &mut deno_runtime.handle_scope();
    deno_value_to_json(scope, res)
}

pub(crate) async fn deno_call_function(
    deno_runtime: &mut deno_core::JsRuntime,
    module: Option<(usize, String)>,
    function: String,
    args: &impl serde::ser::Serialize,
) -> Result<Global<v8::Value>, String> {
    let global_function = deno_get_function_by_name(
        deno_runtime,
        module.clone().map(|e| e.0),
        function.as_str(),
    )?;
    let global_value =
        deno_call_function_by_ref(deno_runtime, module, &global_function, args)?;
    let future = deno_runtime.resolve(global_value);
    let result = deno_runtime
        .with_event_loop_future(future, deno_core::PollEventLoopOptions::default())
        .await
        .map_err(|e| e.to_string());
    result
}

pub(crate) fn deno_value_to_json<'a>(
    scope: &mut HandleScope<'a>,
    result: v8::Global<v8::Value>,
) -> Result<String, String> {
    let result = deno_core::v8::Local::new(scope, result);
    let res = deno_core::serde_v8::from_v8::<deno_core::serde_json::Value>(scope, result)
        .map_err(|e| e.to_string())?;
    let json = format!("{}", res);
    Ok(json)
}

pub(crate) fn deno_get_function_by_name(
    deno_runtime: &mut deno_core::JsRuntime,
    module_id: Option<usize>,
    name: &str,
) -> Result<v8::Global<v8::Function>, String> {
    // Get the value
    let value = deno_get_value_ref(deno_runtime, module_id, name)?;
    let scope = &mut deno_runtime.handle_scope();
    // Convert it into a function
    let local_value = v8::Local::<v8::Value>::new(scope, value);
    let f: v8::Local<v8::Function> = local_value
        .try_into()
        .or::<String>(Err(format!("not_callable {}", name.to_string())))?;
    // Return it as a global
    Ok(v8::Global::<v8::Function>::new(scope, f))
}
pub(crate) fn deno_get_value_ref(
    deno_runtime: &mut deno_core::JsRuntime,
    module_id: Option<usize>,
    name: &str,
) -> Result<v8::Global<v8::Value>, String> {
    // Try to get the value from the module context first
    if let Some(module_id) = module_id {
        return deno_get_module_export_value(deno_runtime, module_id.to_owned(), name);
    }
    // If it's not found, try the global context
    return deno_get_global_value(deno_runtime, name);
}

pub(crate) fn deno_get_module_export_value(
    deno_runtime: &mut deno_core::JsRuntime,
    module_id: usize,
    name: &str,
) -> Result<v8::Global<v8::Value>, String> {
    let module_namespace = deno_runtime
        .get_module_namespace(module_id)
        .map_err(|e| e.to_string())?;
    let scope = &mut deno_runtime.handle_scope();
    let module_namespace = module_namespace.open(scope);
    assert!(module_namespace.is_module_namespace_object());
    let key = name.to_v8_string(scope)?;
    let value = module_namespace.get(scope, key.into());
    match value.if_defined() {
        Some(v) => Ok(v8::Global::<v8::Value>::new(scope, v)),
        _ => Err(format!("module_value_not_found {}", name.to_string())),
    }
}

pub(crate) fn deno_get_global_value(
    deno_runtime: &mut deno_core::JsRuntime,
    name: &str,
) -> Result<v8::Global<v8::Value>, String> {
    let context = deno_runtime.main_context();
    let scope = &mut deno_runtime.handle_scope();
    let global = context.open(scope).global(scope);

    let key = name.to_v8_string(scope)?;
    let value = global.get(scope, key.into());

    match value.if_defined() {
        Some(v) => Ok(v8::Global::<v8::Value>::new(scope, v)),
        _ => Err(format!("global_value_not_found {}", name.to_string())),
    }
}

pub(crate) fn deno_call_function_by_ref(
    deno_runtime: &mut deno_core::JsRuntime,
    module_id: Option<(usize, String)>,
    function: &v8::Global<v8::Function>,
    args: &impl serde::ser::Serialize,
) -> Result<v8::Global<v8::Value>, String> {
    // Namespace, if provided
    let module_namespace = if let Some(module_id) = module_id.clone() {
        Some(
            deno_runtime
                .get_module_namespace(module_id.0)
                .map_err(|e| e.to_string())?,
        )
    } else {
        None
    };

    let scope = &mut deno_runtime.handle_scope();
    let mut scope = v8::TryCatch::new(scope);

    // Get the namespace
    // Module-level if supplied, none otherwise
    let namespace: v8::Local<v8::Value> = if let Some(namespace) = module_namespace {
        v8::Local::<v8::Object>::new(&mut scope, namespace).into()
    } else {
        // Create a new object to use as the namespace if none is provided
        //let obj: v8::Local<v8::Value> = v8::Object::new(scope).into();
        let obj: v8::Local<v8::Value> = v8::undefined(&mut scope).into();
        obj
    };

    let function_instance = function.open(&mut scope);

    // Prep arguments
    let args = deno_decode_args(args, &mut scope)?;

    // Call the function
    let result = function_instance.call(&mut scope, namespace, &args);
    match result {
        Some(value) => {
            let value = v8::Global::new(&mut scope, value);
            Ok(value)
        }
        None if scope.has_caught() => {
            let e = scope
                .message()
                .ok_or_else(|| ("Unknown error".to_string()))?;

            let filename = e.get_script_resource_name(&mut scope);
            let linenumber = e.get_line_number(&mut scope).unwrap_or_default();
            let filename = if let Some(v) = filename {
                let filename = v.to_rust_string_lossy(&mut scope);
                format!("{filename}:{linenumber}: ")
            } else if let Some(module_id) = module_id {
                let filename = module_id.1;
                format!("{filename}:{linenumber}: ")
            } else {
                String::new()
            };

            let msg = e.get(&mut scope).to_rust_string_lossy(&mut scope);
            Err(format!("{filename}{msg}"))
        }
        None => Err("Unknown error during function execution".to_string()),
    }
}

pub(crate) fn deno_decode_args<'a>(
    args: &impl serde::ser::Serialize,
    scope: &mut v8::HandleScope<'a>,
) -> Result<Vec<v8::Local<'a, v8::Value>>, String> {
    let args = deno_core::serde_v8::to_v8(scope, args).map_err(|e| e.to_string())?;
    match v8::Local::<v8::Array>::try_from(args) {
        Ok(args) => {
            let len = args.length();
            let mut result = Vec::with_capacity(len as usize);
            for i in 0..len {
                let index = v8::Integer::new(
                    scope,
                    i.try_into().map_err(|_| {
                        format!("Could not decode {len} arguments - use `big_json_args`")
                    })?,
                );
                let arg = args
                    .get(scope, index.into())
                    .ok_or_else(|| (format!("Invalid argument at index {i}")))?;
                result.push(arg);
            }
            Ok(result)
        }
        Err(_) if args.is_undefined() || args.is_null() => Ok(vec![]),
        Err(_) => Ok(vec![args]),
    }
}

pub(crate) trait ToV8String {
    fn to_v8_string<'a>(
        &self,
        scope: &mut v8::HandleScope<'a>,
    ) -> Result<v8::Local<'a, v8::String>, String>;
}

impl ToV8String for str {
    fn to_v8_string<'a>(
        &self,
        scope: &mut v8::HandleScope<'a>,
    ) -> Result<v8::Local<'a, v8::String>, String> {
        v8::String::new(scope, self).ok_or(format!("to_v8_string_failed {}", self.to_string()))
    }
}

pub(crate) trait ToDefinedValue<T> {
    fn if_defined(&self) -> Option<T>;
}

impl<'a> ToDefinedValue<v8::Local<'a, v8::Value>> for Option<v8::Local<'a, v8::Value>> {
    fn if_defined(&self) -> Option<v8::Local<'a, v8::Value>> {
        self.filter(|v| !v.is_undefined())
    }
}
