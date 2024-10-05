use std::sync::Arc;

use crate::api::deno_commons::DenoCustomData;
use crate::frb_generated::StreamSink;
use deno_core;
use deno_core::v8;
use deno_core::OpState;
use deno_core::{extension, op2};

#[op2(nofast)]
fn op_send_binary(
    state: &mut OpState,
    #[buffer] data: &[u8],
) -> Result<(), deno_core::error::AnyError> {
    if state.has::<Arc<StreamSink<DenoCustomData>>>() {
        let events: Arc<StreamSink<DenoCustomData>> = state.take();
        let data = DenoCustomData {
            binary: Some(data.to_vec()),
            json: None,
        };
        events
            .add(data)
            .map_err(|_| deno_core::error::custom_error("BridgeOps", "event_not_sent"))?;
        Ok(())
    } else {
        Err(deno_core::error::custom_error(
            "BridgeOps",
            "event_not_initiliazed",
        ))
    }
}
#[op2(nofast)]
fn op_send_json(
    state: &mut OpState,
    #[string] data: String,
) -> Result<(), deno_core::error::AnyError> {
    if state.has::<Arc<StreamSink<DenoCustomData>>>() {
        let events: Arc<StreamSink<DenoCustomData>> = state.take();
        let data = DenoCustomData {
            binary: None,
            json: Some(data),
        };
        events
            .add(data)
            .map_err(|_| deno_core::error::custom_error("BridgeOps", "event_not_sent"))?;
        Ok(())
    } else {
        Err(deno_core::error::custom_error(
            "BridgeOps",
            "event_not_initiliazed",
        ))
    }
}
#[derive(Clone, Default, Debug)]
pub(crate) struct Callback {
    pub callbacks: Vec<v8::Global<v8::Function>>,
}

#[op2]
fn op_register_callback(state: &mut OpState, #[global] task: v8::Global<v8::Function>) -> f64 {
    if state.has::<Callback>() {
        let mut all: Callback = state.take();
        all.callbacks.push(task);
        return all.callbacks.len() as f64;
    }
    return 0 as f64;
}

pub fn deno_set_extension_sink(state: &mut OpState, events: Arc<StreamSink<DenoCustomData>>) -> () {
    state.put::<Arc<StreamSink<DenoCustomData>>>(events);
}

pub(crate) fn deno_set_extension_callback(state: &mut OpState) -> () {
    state.put::<Callback>(Callback::default());
}

pub(crate) fn deno_get_extension_callback(state: &mut OpState) -> Option<Callback> {
    if state.has::<Callback>() {
        return Some(state.take::<Callback>());
    }
    None
}

extension!(
    custom_extension,
  deps = [
    deno_webidl,
    deno_console,
    deno_url,
    deno_tls,
    deno_web,
    deno_fetch,
    deno_cache,
    deno_websocket,
    deno_webstorage,
    deno_crypto,
    deno_broadcast_channel,
    deno_node,
    deno_ffi,
    deno_net,
    deno_napi,
    deno_http,
    deno_io,
    deno_fs
  ],
    ops = [op_send_binary, op_send_json, op_register_callback],
    esm_entry_point = "ext:custom_extension/99_main.js",
    esm = [dir "src/ext", "01_errors.js","01_version.js","98_global_scope_shared.js","98_global_scope_window.js", "99_main.js"]
);
