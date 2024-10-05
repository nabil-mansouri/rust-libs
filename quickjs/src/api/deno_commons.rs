use flutter_rust_bridge::frb;

#[frb(external)]
#[frb(non_opaque)]
#[derive(Clone, Debug)]
pub struct DenoCustomData {
    pub binary: Option<Vec<u8>>,
    pub json: Option<String>,
}

#[frb(external)]
#[frb(non_opaque)]
pub struct DenoMemoryUsage {
    pub total_available_size: usize,
    pub total_physical_size: usize,
    pub total_heap_size: usize,
    pub used_heap_size: usize,
    pub external_memory: usize,
}
#[macro_export]
macro_rules! json_args_custom {
    ($($arg:expr),*) => {
        &($($arg),*)
    };
}

#[allow(dead_code)]
pub(crate) fn deno_tokio_current_thread() -> Result<tokio::runtime::Runtime, String> {
    return tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string());
} 
#[allow(dead_code)]
pub(crate) fn deno_tokio_multi_thread() -> Result<tokio::runtime::Runtime, String> {
    return tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string());
}
