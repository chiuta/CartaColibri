//! JSON API behind the chat UI (/carta/...): the Library page, RAG-fetch and RAG-CARTA.
//! One Library (one SQLite connection) per request — connections are never shared between threads.

use crate::{
    carta, grab,
    library::{Filter, Library},
};
use serde_json::{Value, json};
use std::collections::HashMap;

pub struct Req<'a> {
    pub method: &'a str,
    pub route: &'a str,
    pub query: HashMap<String, String>,
    pub body: &'a [u8],
    pub filename: Option<String>,
    pub backend_url: &'a str,
}

fn jbody(b: &[u8]) -> Result<Value, String> {
    if b.is_empty() {
        Ok(json!({}))
    } else {
        serde_json::from_slice(b).map_err(|e| format!("bad JSON: {e}"))
    }
}

fn s<'a>(j: &'a Value, k: &str) -> Result<&'a str, String> {
    j[k].as_str().ok_or_else(|| format!("missing '{k}'"))
}

fn strs(j: &Value, k: &str) -> Vec<String> {
    j[k].as_array()
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn csv(q: &HashMap<String, String>, k: &str) -> Vec<String> {
    q.get(k)
        .map(|v| {
            v.split(',')
                .filter(|x| !x.is_empty())
                .map(String::from)
                .collect()
        })
        .unwrap_or_default()
}

pub fn handle(home: Option<&str>, r: &Req) -> (u16, Value) {
    match inner(home, r) {
        Ok(v) => (200, v),
        Err((code, m)) => (code, json!({"error": m})),
    }
}

fn inner(home: Option<&str>, r: &Req) -> Result<Value, (u16, String)> {
    let bad = |m: String| (400u16, m);
    let mut lib = Library::open(home, false).map_err(|m| (500, m))?;
    let (m, route) = (r.method, r.route);
    Ok(match (m, route) {
        ("GET", "library") => {
            json!({"roots": lib.roots(), "documents": lib.documents(None).map_err(bad)?, "groups": lib.groups(),
                                     "problems": lib.problems(), "home": lib.home})
        }
        ("POST", "folders") => {
            let j = jbody(r.body).map_err(bad)?;
            let p = s(&j, "path").map_err(bad)?.to_string();
            lib.add_root(&p).map_err(bad)?;
            serde_json::to_value(lib.update(Some(&[p]), &mut |_| {}).map_err(bad)?).unwrap()
        }
        ("DELETE", "folders") => {
            let j = jbody(r.body).map_err(bad)?;
            json!({"dropped": lib.remove_root(s(&j, "path").map_err(bad)?).map_err(bad)?})
        }
        ("POST", "upload") => lib
            .add_upload(
                r.filename.as_deref().unwrap_or("document"),
                r.body,
                &csv(&r.query, "ocr"),
            )
            .map_err(bad)?,
        ("POST", "grab") => {
            let j = jbody(r.body).map_err(bad)?;
            let url = s(&j, "url").map_err(bad)?;
            match j["text"].as_str() {
                Some(t) => {
                    grab::save(&mut lib, url, j["title"].as_str().unwrap_or(""), t).map_err(bad)?
                }
                None => grab::grab(&mut lib, url).map_err(|m| (502, m))?,
            }
        }
        ("GET", "status") => {
            json!({"ocr": crate::extract::ocr_available(), "home": lib.home, "version": 2})
        }
        ("POST", "index") => {
            serde_json::to_value(lib.update(None, &mut |_| {}).map_err(bad)?).unwrap()
        }
        ("GET", "search") => {
            let f = Filter {
                ext: csv(&r.query, "ext"),
                min_size: r.query.get("min").and_then(|v| v.parse().ok()),
                max_size: r.query.get("max").and_then(|v| v.parse().ok()),
                path_like: r.query.get("path").cloned().filter(|p| !p.is_empty()),
                group: r.query.get("group").cloned().filter(|p| !p.is_empty()),
                limit: None,
            };
            serde_json::to_value(
                lib.search(r.query.get("q").map(String::as_str).unwrap_or(""), &f)
                    .map_err(bad)?,
            )
            .unwrap()
        }
        ("GET", "doc") => {
            let id: i64 = r
                .query
                .get("id")
                .and_then(|v| v.parse().ok())
                .ok_or_else(|| bad("missing 'id'".into()))?;
            json!({"text": lib.text_of(id)})
        }
        ("POST", "groups") => {
            let j = jbody(r.body).map_err(bad)?;
            serde_json::to_value(
                lib.group_set(
                    s(&j, "name").map_err(bad)?,
                    &strs(&j, "members"),
                    j["description"].as_str(),
                    j["add"].as_bool().unwrap_or(false),
                )
                .map_err(bad)?,
            )
            .unwrap()
        }
        ("DELETE", "groups") => {
            let j = jbody(r.body).map_err(bad)?;
            lib.group_remove(s(&j, "name").map_err(bad)?, &strs(&j, "members"))
                .map_err(bad)?;
            json!({"ok": true})
        }
        ("POST", "run") => {
            let j = jbody(r.body).map_err(bad)?;
            let task = s(&j, "task").map_err(bad)?;
            let docs = match j["group"].as_str().filter(|g| !g.is_empty()) {
                Some(g) => lib.resolve_group(g).map_err(bad)?,
                None => lib.docs_by_id(
                    &j["ids"]
                        .as_array()
                        .map(|a| a.iter().filter_map(Value::as_i64).collect::<Vec<_>>())
                        .unwrap_or_default(),
                ),
            };
            let be = if j["mock"].as_bool().unwrap_or(false) {
                carta::Backend::mock()
            } else {
                carta::Backend::gateway(r.backend_url, None)
            };
            let request = j["request"].as_str().unwrap_or("");
            let o = carta::Options {
                request,
                fields: strs(&j, "fields"),
                json: j["fmt"].as_str() == Some("json"),
                budget: carta::CTX_CHARS,
            };
            let text = carta::run(&lib, task, &docs, &be, &o, &mut |_| {}).map_err(|m| {
                if m.starts_with("no answer from the Colibr") {
                    (502, m)
                } else {
                    (400, m)
                }
            })?;
            let saved = if j["save"].as_bool().unwrap_or(false) {
                Some(
                    carta::save(
                        &lib.home,
                        &text,
                        task,
                        j["group"].as_str().unwrap_or("selection"),
                        request,
                    )
                    .map_err(bad)?,
                )
            } else {
                None
            };
            json!({"text": text, "saved": saved})
        }
        _ => return Err((404, "no such carta route".into())),
    })
}
