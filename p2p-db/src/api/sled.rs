use std::{ops::Deref, sync::Arc};

use flutter_rust_bridge::{frb, DartFnFuture};
use sled::{Batch, IVec};

#[frb(external)]
#[frb(opaque)]
pub struct CustomDB {
    db: sled::Db,
}

#[frb(external)]
#[frb(opaque)]
pub struct CustomBatch {
    pub upserts: Vec<(Vec<u8>, Vec<u8>)>,
    pub deletes: Vec<Vec<u8>>,
}

#[frb(sync)]
pub fn sled_db_key_from_string(key: String) -> Vec<u8> {
    return key.as_bytes().to_vec();
}

#[frb(sync)]
pub fn sled_db_key_to_string(key: Vec<u8>) -> String {
    return String::from_utf8_lossy(key.as_ref()).to_string();
}

pub async fn sled_db_open(
    path: String,
    compression: bool,
    temporary: bool,
) -> Result<Arc<CustomDB>, String> {
    let db = sled::Config::new()
        .path(path)
        .use_compression(compression)
        .temporary(temporary)
        .open()
        .map_err(|e| e.to_string())?;
    let res = Arc::new(CustomDB { db });
    Ok(res)
}

pub async fn sled_db_delete_all(db: &Arc<CustomDB>, tree: Option<String>) -> Result<(), String> {
    let tree_safe = get_tree(db, tree)?;
    tree_safe.clear().map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn sled_db_drop(db: &Arc<CustomDB>, tree: Option<String>) -> Result<bool, String> {
    if let Some(tree_name) = tree {
        let tree = db.db.drop_tree(tree_name).map_err(|e| e.to_string())?;
        Ok(tree)
    } else {
        let tree = db.db.drop_tree(b"__sled__default").map_err(|e| e.to_string())?;
        Ok(tree)
    }
}

pub async fn sled_db_count(db: &Arc<CustomDB>, tree: Option<String>) -> Result<usize, String> {
    let tree_safe = get_tree(db, tree)?;
    let len = tree_safe.len();
    Ok(len)
}

pub async fn sled_db_upsert(
    db: &Arc<CustomDB>,
    key: Vec<u8>,
    value: Vec<u8>,
    tree: Option<String>,
) -> Result<Option<Vec<u8>>, String> {
    let tree_safe = get_tree(db, tree)?;
    let res = tree_safe.insert(key, value).map_err(|e| e.to_string())?;
    Ok(res.map(|e| e.to_vec()))
}

pub async fn sled_db_delete(
    db: &Arc<CustomDB>,
    key: Vec<u8>,
    tree: Option<String>,
) -> Result<Option<Vec<u8>>, String> {
    let tree_safe = get_tree(db, tree)?;
    let res = tree_safe.remove(key).map_err(|e| e.to_string())?;
    Ok(res.map(|e| e.to_vec()))
}

pub async fn sled_db_get(
    db: &Arc<CustomDB>,
    key: Vec<u8>,
    tree: Option<String>,
) -> Result<Option<Vec<u8>>, String> {
    let tree_safe = get_tree(db, tree)?;
    let res = tree_safe.get(key).map_err(|e| e.to_string())?;
    Ok(res.map(|e| e.to_vec()))
}

pub async fn sled_db_get_previous(
    db: &Arc<CustomDB>,
    key: Vec<u8>,
    tree: Option<String>,
) -> Result<Option<(Vec<u8>, Vec<u8>)>, String> {
    let tree_safe = get_tree(db, tree)?;
    let res = tree_safe.get_lt(key).map_err(|e| e.to_string())?;
    Ok(res.map(|(key, value)| (key.to_vec(), value.to_vec())))
}

pub async fn sled_db_get_next(
    db: &Arc<CustomDB>,
    key: Vec<u8>,
    tree: Option<String>,
) -> Result<Option<(Vec<u8>, Vec<u8>)>, String> {
    let tree_safe = get_tree(db, tree)?;
    let res = tree_safe.get_gt(key).map_err(|e| e.to_string())?;
    Ok(res.map(|(key, value)| (key.to_vec(), value.to_vec())))
}

pub async fn sled_db_contains(
    db: &Arc<CustomDB>,
    key: Vec<u8>,
    tree: Option<String>,
) -> Result<bool, String> {
    let tree_safe = get_tree(db, tree)?;
    let res = tree_safe.contains_key(key).map_err(|e| e.to_string())?;
    Ok(res)
}

pub async fn sled_db_key_range(
    db: &Arc<CustomDB>,
    start: Option<String>,
    end: Option<String>,
    limit: Option<usize>,
    tree: Option<String>,
) -> Result<Vec<Vec<u8>>, String> {
    let tree_safe = get_tree(db, tree)?;
    let range_result: Result<Vec<Vec<u8>>, _> = match (start.as_ref(), end.as_ref()) {
        (Some(start), Some(end)) => convert_keys_to_bytes(
            tree_safe.range(start.as_bytes()..end.as_bytes()).keys(),
            limit,
        ),
        (Some(start), None) => {
            convert_keys_to_bytes(tree_safe.range(start.as_bytes()..).keys(), limit)
        }
        (None, Some(end)) => convert_keys_to_bytes(tree_safe.range(..end.as_bytes()).keys(), limit),
        (None, None) => convert_keys_to_bytes(tree_safe.iter().keys(), limit),
    };
    let keys = range_result.map_err(|e| e.to_string())?;
    Ok(keys)
}

pub async fn sled_db_key_value_range(
    db: &Arc<CustomDB>,
    start: Option<String>,
    end: Option<String>,
    limit: Option<usize>,
    tree: Option<String>,
) -> Result<Vec<(Vec<u8>, Vec<u8>)>, String> {
    let tree_safe = get_tree(db, tree)?;
    let range_result: Result<Vec<(Vec<u8>, Vec<u8>)>, _> = match (start.as_ref(), end.as_ref()) {
        (Some(start), Some(end)) => {
            convert_keys_values_to_bytes(tree_safe.range(start.as_bytes()..end.as_bytes()), limit)
        }
        (Some(start), None) => {
            convert_keys_values_to_bytes(tree_safe.range(start.as_bytes()..), limit)
        }
        (None, Some(end)) => convert_keys_values_to_bytes(tree_safe.range(..end.as_bytes()), limit),
        (None, None) => convert_keys_values_to_bytes(tree_safe.iter(), limit),
    };
    let keys = range_result.map_err(|e| e.to_string())?;
    Ok(keys)
}

pub async fn sled_db_key_range_fn(
    db: &Arc<CustomDB>,
    start: Option<String>,
    end: Option<String>,
    tree: Option<String>,
    callback: impl Fn(Vec<u8>) -> DartFnFuture<bool> + Send + 'static,
) -> Result<(), String> {
    let tree_safe = get_tree(db, tree)?;
    let range_result: Result<(), _> = match (start.as_ref(), end.as_ref()) {
        (Some(start), Some(end)) => {
            convert_keys_to_bytes_fn(
                tree_safe.range(start.as_bytes()..end.as_bytes()).keys(),
                callback,
            )
            .await
        }
        (Some(start), None) => {
            convert_keys_to_bytes_fn(tree_safe.range(start.as_bytes()..).keys(), callback).await
        }
        (None, Some(end)) => {
            convert_keys_to_bytes_fn(tree_safe.range(..end.as_bytes()).keys(), callback).await
        }
        (None, None) => convert_keys_to_bytes_fn(tree_safe.iter().keys(), callback).await,
    };
    let keys = range_result.map_err(|e| e.to_string())?;
    Ok(keys)
}

pub async fn sled_db_key_value_range_fn(
    db: &Arc<CustomDB>,
    start: Option<String>,
    end: Option<String>,
    tree: Option<String>,
    callback: impl Fn(Vec<u8>, Vec<u8>) -> DartFnFuture<bool> + Send + 'static,
) -> Result<(), String> {
    let tree_safe = get_tree(db, tree)?;
    let range_result = match (start.as_ref(), end.as_ref()) {
        (Some(start), Some(end)) => {
            convert_keys_values_to_bytes_fn(
                tree_safe.range(start.as_bytes()..end.as_bytes()),
                callback,
            )
            .await
        }
        (Some(start), None) => {
            convert_keys_values_to_bytes_fn(tree_safe.range(start.as_bytes()..), callback).await
        }
        (None, Some(end)) => {
            convert_keys_values_to_bytes_fn(tree_safe.range(..end.as_bytes()), callback).await
        }
        (None, None) => convert_keys_values_to_bytes_fn(tree_safe.iter(), callback).await,
    };
    let keys = range_result.map_err(|e| e.to_string())?;
    Ok(keys)
}

pub async fn sled_db_key_prefix(
    db: &Arc<CustomDB>,
    prefix: String,
    limit: Option<usize>,
    tree: Option<String>,
) -> Result<Vec<Vec<u8>>, String> {
    let tree_safe = get_tree(db, tree)?;
    let range_result: Result<Vec<Vec<u8>>, _> =
        convert_keys_to_bytes(tree_safe.scan_prefix(prefix).keys(), limit);
    let keys = range_result.map_err(|e| e.to_string())?;
    Ok(keys)
}

pub async fn sled_db_key_value_prefix(
    db: &Arc<CustomDB>,
    prefix: String,
    limit: Option<usize>,
    tree: Option<String>,
) -> Result<Vec<(Vec<u8>, Vec<u8>)>, String> {
    let tree_safe = get_tree(db, tree)?;
    let range_result: Result<Vec<(Vec<u8>, Vec<u8>)>, _> =
        convert_keys_values_to_bytes(tree_safe.scan_prefix(prefix), limit);
    let keys = range_result.map_err(|e| e.to_string())?;
    Ok(keys)
}

pub async fn sled_db_key_prefix_fn(
    db: &Arc<CustomDB>,
    prefix: String,
    tree: Option<String>,
    callback: impl Fn(Vec<u8>) -> DartFnFuture<bool> + Send + 'static,
) -> Result<(), String> {
    let tree_safe = get_tree(db, tree)?;
    let range_result: Result<(), _> =
        convert_keys_to_bytes_fn(tree_safe.scan_prefix(prefix).keys(), callback).await;
    let keys = range_result.map_err(|e| e.to_string())?;
    Ok(keys)
}

pub async fn sled_db_key_value_prefix_fn(
    db: &Arc<CustomDB>,
    prefix: String,
    tree: Option<String>,
    callback: impl Fn(Vec<u8>, Vec<u8>) -> DartFnFuture<bool> + Send + 'static,
) -> Result<(), String> {
    let tree_safe = get_tree(db, tree)?;
    let range_result: Result<(), _> =
        convert_keys_values_to_bytes_fn(tree_safe.scan_prefix(prefix), callback).await;
    let keys = range_result.map_err(|e| e.to_string())?;
    Ok(keys)
}

pub async fn sled_db_flush(db: &Arc<CustomDB>, tree: Option<String>) -> Result<usize, String> {
    let tree_safe = get_tree(db, tree)?;
    let size = tree_safe.flush_async().await.map_err(|e| e.to_string())?;
    Ok(size)
}
pub async fn sled_db_close(db: &Arc<CustomDB>) -> Result<usize, String> {
    let size = db.db.flush_async().await.map_err(|e| e.to_string())?;
    drop(db.to_owned());
    Ok(size)
}

#[frb(sync)]
pub fn sled_db_transaction_begin() -> CustomBatch {
    return CustomBatch {
        deletes: Vec::new(),
        upserts: Vec::new(),
    };
}

pub async fn sled_db_transaction_commit(
    db: &Arc<CustomDB>,
    batch: &CustomBatch,
    tree: Option<String>,
) -> Result<(), String> {
    let tree_safe = get_tree(db, tree)?;
    let mut db_batch = Batch::default();
    // delete first (dont delete new insert)
    for key in batch.deletes.as_slice() {
        db_batch.remove(key.as_slice());
    }
    // then insert
    for (key, value) in batch.upserts.as_slice() {
        db_batch.insert(key.as_slice(), value.as_slice());
    }
    tree_safe.apply_batch(db_batch).map_err(|e| e.to_string())?;
    Ok(())
}

fn convert_keys_to_bytes(
    iterator: impl DoubleEndedIterator<Item = sled::Result<IVec>> + Send + Sync,
    limit: Option<usize>,
) -> Result<Vec<Vec<u8>>, sled::Error> {
    let real_limit = limit.unwrap_or_else(|| 0);
    let mut result: Vec<Vec<u8>> = Vec::new();
    for current in iterator {
        let key = current?;
        if 0 == real_limit || result.len() < real_limit {
            let to_add = key.to_vec();
            result.push(to_add);
        } else {
            break;
        }
    }
    Ok(result)
}

async fn convert_keys_to_bytes_fn(
    iterator: impl DoubleEndedIterator<Item = sled::Result<IVec>> + Send + Sync,
    callback: impl Fn(Vec<u8>) -> DartFnFuture<bool> + Send + 'static,
) -> Result<(), sled::Error> {
    for current in iterator {
        let key = current?;
        let should_continue = callback(key.to_vec()).await;
        if !should_continue {
            break;
        }
    }
    Ok(())
}

fn convert_keys_values_to_bytes(
    iterator: impl DoubleEndedIterator<Item = sled::Result<(IVec, IVec)>> + Send + Sync,
    limit: Option<usize>,
) -> Result<Vec<(Vec<u8>, Vec<u8>)>, sled::Error> {
    let real_limit = limit.unwrap_or_else(|| 0);
    let mut result: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
    for current in iterator {
        let (key, value) = current?;
        if 0 == real_limit || result.len() < real_limit {
            let to_add = (key.to_vec(), value.to_vec());
            result.push(to_add);
        } else {
            break;
        }
    }
    Ok(result)
}

async fn convert_keys_values_to_bytes_fn(
    iterator: impl DoubleEndedIterator<Item = sled::Result<(IVec, IVec)>> + Send + Sync,
    callback: impl Fn(Vec<u8>, Vec<u8>) -> DartFnFuture<bool> + Send + 'static,
) -> Result<(), sled::Error> {
    for current in iterator {
        let (key, value) = current?;
        let should_continue = callback(key.to_vec(), value.to_vec()).await;
        if !should_continue {
            break;
        }
    }
    Ok(())
}
fn get_tree(db: &Arc<CustomDB>, tree: Option<String>) -> Result<sled::Tree, String> {
    if let Some(tree_name) = tree {
        let tree = db.db.open_tree(tree_name).map_err(|e| e.to_string())?;
        Ok(tree)
    } else {
        let tree = db.db.deref().to_owned();
        Ok(tree)
    }
}
