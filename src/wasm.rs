use js_sys::{Array, Object, Reflect, Uint8Array};
use wasm_bindgen::prelude::*;

use crate::CompressedTrainingDataEntryReader;

fn set_property(target: &Object, key: &str, value: JsValue) -> Result<(), JsValue> {
    Reflect::set(target, &JsValue::from_str(key), &value).map(|_| ())
}

#[wasm_bindgen]
pub fn parse_binpack_chunk(bytes: Uint8Array, preview_limit: usize) -> Result<Object, JsValue> {
    let payload = bytes.to_vec();
    let chunk_size = payload.len() as u32;

    let mut file_bytes = Vec::with_capacity(payload.len() + 8);
    file_bytes.extend_from_slice(b"BINP");
    file_bytes.extend_from_slice(&chunk_size.to_le_bytes());
    file_bytes.extend_from_slice(&payload);

    let mut reader = CompressedTrainingDataEntryReader::from_bytes(file_bytes)
        .map_err(|err| JsValue::from_str(&err.to_string()))?;

    let result = Object::new();
    let preview = Array::new();
    let mut entries_read = 0u32;

    while reader.has_next() && entries_read < preview_limit as u32 {
        let continuation = reader.is_next_entry_continuation();
        let entry = reader.next();

        let js_entry = Object::new();

        set_property(
            &js_entry,
            "fen",
            JsValue::from_str(&entry.pos.fen().unwrap()),
        )?;
        set_property(&js_entry, "uci", JsValue::from_str(&entry.mv.as_uci()))?;
        set_property(&js_entry, "score", JsValue::from_f64(entry.score as f64))?;
        set_property(&js_entry, "ply", JsValue::from_f64(entry.ply as f64))?;
        set_property(&js_entry, "result", JsValue::from_f64(entry.result as f64))?;
        set_property(&js_entry, "continuation", JsValue::from_bool(continuation))?;

        preview.push(&js_entry);

        entries_read += 1;
    }

    set_property(
        &result,
        "byteLength",
        JsValue::from_f64(payload.len() as f64),
    )?;
    set_property(
        &result,
        "entriesRead",
        JsValue::from_f64(preview.length() as f64),
    )?;
    set_property(&result, "preview", preview.into())?;

    Ok(result)
}
