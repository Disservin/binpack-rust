use js_sys::{Array, Object, Reflect, Uint8Array};
use wasm_bindgen::prelude::*;

use crate::CompressedTrainingDataEntryReader;

fn set_property(target: &Object, key: &str, value: JsValue) -> Result<(), JsValue> {
    Reflect::set(target, &JsValue::from_str(key), &value).map(|_| ())
}

#[wasm_bindgen]
pub fn parse_binpack(bytes: Uint8Array, preview_limit: usize) -> Result<Object, JsValue> {
    let mut reader = CompressedTrainingDataEntryReader::from_bytes(bytes.to_vec())
        .map_err(|err| JsValue::from_str(&err.to_string()))?;

    let result = Object::new();
    let preview = Array::new();
    let mut total_entries = 0u32;

    while reader.has_next() {
        let continuation = reader.is_next_entry_continuation();
        let entry = reader.next();

        if total_entries < preview_limit as u32 {
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
        }

        total_entries += 1;
    }

    set_property(
        &result,
        "byteLength",
        JsValue::from_f64(bytes.length() as f64),
    )?;
    set_property(
        &result,
        "previewCount",
        JsValue::from_f64(preview.length() as f64),
    )?;
    set_property(
        &result,
        "totalEntries",
        JsValue::from_f64(total_entries as f64),
    )?;
    set_property(&result, "preview", preview.into())?;

    Ok(result)
}
