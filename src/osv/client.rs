use serde_json::{json, Value};
use std::{thread, time::Duration};

pub(crate) fn post_osv(url: &str, body: &Value) -> Result<Value, String> {
    let mut last_err = String::new();
    for attempt in 0..2 {
        match crate::http::agent().post(url).send_json(body) {
            Ok(mut resp) => {
                return resp.body_mut().read_json::<Value>().map_err(|e| format!("OSV invalid JSON: {e}"));
            }
            Err(e) => {
                last_err = format!("OSV request failed: {e}");
                if attempt == 0 { thread::sleep(Duration::from_millis(300)); }
            }
        }
    }
    Err(last_err)
}

pub fn query_batch(queries: &[(String, String, String)]) -> Result<Vec<Vec<String>>, String> {
    let mut all = Vec::with_capacity(queries.len());
    for chunk in queries.chunks(500) {
        let payload: Vec<Value> = chunk
            .iter()
            .map(|(e, n, v)| json!({ "package": { "name": n, "ecosystem": e }, "version": v }))
            .collect();
        let data = post_osv("https://api.osv.dev/v1/querybatch", &json!({ "queries": payload }))?;
        let results = data.get("results").and_then(Value::as_array).ok_or("OSV missing results")?;
        for res in results {
            let ids = res.get("vulns").and_then(Value::as_array)
                .map(|a| a.iter().filter_map(|v| v.get("id").and_then(Value::as_str).map(String::from)).collect())
                .unwrap_or_default();
            all.push(ids);
        }
    }
    Ok(all)
}
