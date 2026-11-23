//! web-grab browser — a small Rust browser on the system webview (WebKitGTK / WebView2 / WKWebView).
//!
//! Beyond tabs, it is a browser that knows your library:
//! * 📚 a badge when the page is already saved, with a warning when it differs from the saved copy
//! * 🔗 related documents from your library while you browse (RAG-fetch on the page title)
//! * 💾 save page / selection (the address stays as the source; saving again refreshes the same file)
//! * ✨ ask your local Colibrì about the page — or 🗂✨ across ALL open tabs, with [S#] citations per tab
//! * 🗂 save every open tab as one library group; words in the address bar search your library
//! * the session (open tabs) is restored next time
//!
//! Clean-room, MIT. It reuses no Falkon (GPL-3.0) or other browser code. Nothing leaves your machine
//! except the page fetches you ask for; the bar talks to the app through a per-run nonce.

use cartacolibri::carta::{self, Backend, Options};
use cartacolibri::grab;
use cartacolibri::library::{Filter, Library, state_dir};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use tao::event::{Event, StartCause, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy};
use tao::window::{Window, WindowBuilder};
use wry::{WebView, WebViewBuilder};

const OVERLAY: &str = include_str!("overlay.js");
const START: &str = r#"<!doctype html><meta charset=utf-8><title>CartaColibri web-grab</title>
<body style="font:18px system-ui;background:#0f1115;color:#eef1f6;display:grid;place-content:center;height:80vh;text-align:center">
<h1 style="color:#5fd7e6">CartaColibri · web-grab</h1>
<p>Type an address in the bar, or words to search your library.<br>Open pages in tabs, press <b>💾</b> to keep one or <b>✨</b> to ask about it —<br><b>🗂✨</b> asks across all your open tabs.</p></body>"#;

enum Ev {
    Eval(usize, String),
    Navigate(usize, String),
    NewTab(Option<String>),
    Close(usize),
    Activate(usize),
    Cycle(bool),
    Title(usize, String),
    Url(usize, String),
}

#[derive(Clone, Default)]
struct Page {
    url: String,
    title: String,
    text: String,
}

struct Cfg {
    home: Option<String>,
    gateway: String,
    mock: bool,
    nonce: String,
    selftest: Option<&'static str>,
    pages: Mutex<BTreeMap<usize, Page>>,
}

struct Tab {
    id: usize,
    wv: WebView,
    title: String,
    url: String,
}

fn nonce() -> String {
    use sha1::{Digest, Sha1};
    let mut seed = format!("{:?}{}", std::time::SystemTime::now(), std::process::id());
    if let Ok(mut f) = std::fs::File::open("/dev/urandom") {
        use std::io::Read;
        let mut b = [0u8; 16];
        if f.read_exact(&mut b).is_ok() {
            seed.push_str(&hex::encode(b));
        }
    }
    hex::encode(Sha1::digest(seed.as_bytes()))
}

fn normalize(q: &str) -> Option<String> {
    let q = q.trim();
    if q.is_empty() || q.contains(' ') {
        return None;
    }
    if q.starts_with("http://") || q.starts_with("https://") {
        return Some(q.to_string());
    }
    if q.contains('.') || q.starts_with("localhost") {
        return Some(format!("https://{q}"));
    }
    None
}

fn reply(p: &EventLoopProxy<Ev>, c: &Cfg, tab: usize, ok: bool, text: String) {
    let msg = json!({"ok": ok, "text": text});
    let js = format!(
        "window[\"__cc_{n}\"]&&window[\"__cc_{n}\"]({msg})",
        n = c.nonce
    );
    let _ = p.send_event(Ev::Eval(tab, js));
}

fn backend(c: &Cfg) -> Backend {
    if c.mock {
        Backend::mock()
    } else {
        Backend::gateway(&c.gateway, None)
    }
}

/// Save every open tab (that has readable text) and group them. Returns (group name, saved paths).
fn save_tabs(c: &Cfg, lib: &mut Library) -> Result<(String, Vec<String>), String> {
    let pages: Vec<Page> = c.pages.lock().unwrap().values().cloned().collect();
    let mut paths = vec![];
    for p in pages.iter().filter(|p| p.url.starts_with("http")) {
        if let Ok(v) = grab::save(lib, &p.url, &p.title, &p.text)
            && let Some(path) = v["saved"].as_str()
        {
            paths.push(path.to_string());
        }
    }
    if paths.is_empty() {
        return Err("no open tab has readable text yet".into());
    }
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let name = format!("tabs-{secs}");
    lib.group_set(&name, &paths, Some("web-grab: open tabs"), false)?;
    Ok((name, paths))
}

fn related(lib: &Library, title: &str, skip: Option<&str>) -> Vec<Value> {
    let words: Vec<String> = title
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.chars().count() >= 4)
        .take(6)
        .map(String::from)
        .collect();
    if words.is_empty() {
        return vec![];
    }
    lib.search(
        &words.join(" OR "),
        &Filter {
            limit: Some(5),
            ..Default::default()
        },
    )
    .unwrap_or_default()
    .into_iter()
    .filter(|h| Some(h.path.as_str()) != skip)
    .take(4)
    .map(|h| json!({"path": h.path, "snippet": h.snippet, "score": h.score}))
    .collect()
}

fn handle(c: &Cfg, p: &EventLoopProxy<Ev>, m: &Value) {
    let s = |k: &str| m[k].as_str().unwrap_or("").to_string();
    let tab = m["t"].as_u64().unwrap_or(0) as usize;
    let lib = || Library::open(c.home.as_deref(), false);
    match m["cmd"].as_str().unwrap_or("") {
        "newtab" => {
            let _ = p.send_event(Ev::NewTab(None));
        }
        "close" => {
            let _ = p.send_event(Ev::Close(m["id"].as_u64().unwrap_or(0) as usize));
        }
        "activate" => {
            let _ = p.send_event(Ev::Activate(m["id"].as_u64().unwrap_or(0) as usize));
        }
        "cycle" => {
            let _ = p.send_event(Ev::Cycle(m["back"].as_bool().unwrap_or(false)));
        }
        "ready" => {}
        "loaded" => {
            let page = Page {
                url: s("url"),
                title: s("title"),
                text: s("text"),
            };
            c.pages.lock().unwrap().insert(tab, page.clone());
            let _ = p.send_event(Ev::Url(tab, page.url.clone()));
            let _ = p.send_event(Ev::Title(tab, page.title.clone()));
            if !page.url.starts_with("http") {
                return;
            }
            let home = state_dir(c.home.as_deref()).ok();
            let status = home
                .as_ref()
                .and_then(|h| grab::status(h, &page.url, &page.text));
            let skip = status
                .as_ref()
                .and_then(|s| s["path"].as_str().map(String::from));
            let history = home.as_ref().is_some_and(|h| grab::history_enabled(h));
            let mut rel = lib()
                .map(|l| related(&l, &page.title, skip.as_deref()))
                .unwrap_or_default();
            if let Some(h) = home.as_ref() {
                let own = grab::history_path(h, &page.url)
                    .to_string_lossy()
                    .into_owned();
                rel.retain(|r| r["path"].as_str() != Some(own.as_str()));
            }
            let hl = home
                .as_ref()
                .map(|h| grab::highlights(h, &page.url))
                .unwrap_or_default();
            let info =
                json!({"status": status, "related": rel, "highlights": hl, "history": history});
            let js = format!(
                "window[\"__cc_info_{n}\"]&&window[\"__cc_info_{n}\"]({info})",
                n = c.nonce
            );
            let _ = p.send_event(Ev::Eval(tab, js));
            if let (true, Ok(mut l)) = (history, lib()) {
                let _ = grab::history_add(&mut l, &page.url, &page.title, &page.text);
            }
        }
        "go" if s("q").trim_start().starts_with('?') => {
            let q = s("q")
                .trim_start()
                .trim_start_matches('?')
                .trim()
                .to_string();
            let r = lib().and_then(|l| {
                let hits = l.search(
                    &q,
                    &Filter {
                        limit: Some(8),
                        ..Default::default()
                    },
                )?;
                let ids: Vec<i64> = hits.iter().map(|h| h.id).collect();
                let docs = l.docs_by_id(&ids);
                let o = Options {
                    request: &q,
                    ..Default::default()
                };
                carta::run(&l, "synthesize", &docs, &backend(c), &o, &mut |_| {})
            });
            match r {
                Ok(t) => reply(p, c, tab, true, t),
                Err(e) => reply(p, c, tab, false, e),
            }
        }
        "go" => match normalize(&s("q")) {
            Some(u) => {
                let _ = p.send_event(Ev::Navigate(tab, u));
            }
            None => {
                let out = lib()
                    .and_then(|l| {
                        l.search(
                            &s("q"),
                            &Filter {
                                limit: Some(10),
                                ..Default::default()
                            },
                        )
                    })
                    .map(|hits| {
                        if hits.is_empty() {
                            "—".to_string()
                        } else {
                            hits.iter()
                                .map(|h| {
                                    format!("{:>3}%  {}\n      {}", h.score, h.path, h.snippet)
                                })
                                .collect::<Vec<_>>()
                                .join("\n")
                        }
                    });
                match out {
                    Ok(t) => reply(p, c, tab, true, t),
                    Err(e) => reply(p, c, tab, false, e),
                }
            }
        },
        "save" => {
            match lib().and_then(|mut l| {
                grab::save_with(
                    &mut l,
                    &s("url"),
                    &s("title"),
                    &s("text"),
                    m["html"].as_str(),
                )
            }) {
                Ok(v) => reply(
                    p,
                    c,
                    tab,
                    true,
                    format!(
                        "✔ {} characters → {}",
                        v["chars"],
                        v["saved"].as_str().unwrap_or("")
                    ),
                ),
                Err(e) => reply(p, c, tab, false, e),
            }
        }
        "highlight" => {
            let r = lib().and_then(|mut l| {
                if grab::status(&l.home, &s("url"), "").is_none() {
                    grab::save(&mut l, &s("url"), &s("title"), &s("text"))?;
                }
                grab::highlight_add(&mut l, &s("url"), &s("sel"))
            });
            match r {
                Ok(n) => reply(
                    p,
                    c,
                    tab,
                    true,
                    format!("🖍 {n} highlight(s) saved with this page"),
                ),
                Err(e) => reply(p, c, tab, false, e),
            }
        }
        "watch" => {
            let url = s("url");
            if let Ok(h) = state_dir(c.home.as_deref()) {
                let mut l = grab::watch_list(&h);
                if !l.contains(&url) {
                    l.push(url.clone());
                }
                grab::watch_set(&h, l);
                reply(
                    p,
                    c,
                    tab,
                    true,
                    format!(
                        "👁 watching {url}\ncheck with: cartacolibri webwatch check   (or run, to keep checking)"
                    ),
                );
            }
        }
        "history" => {
            if let Ok(h) = state_dir(c.home.as_deref()) {
                let on = !grab::history_enabled(&h);
                grab::set_history(&h, on);
                reply(
                    p,
                    c,
                    tab,
                    true,
                    format!(
                        "🕘 history {} — {}",
                        if on { "on" } else { "off" },
                        if on {
                            "pages you visit are kept locally and searchable from the address bar"
                        } else {
                            "nothing new is recorded"
                        }
                    ),
                );
                let js = format!(
                    "window[\"__cc_info_{n}\"]&&window[\"__cc_info_{n}\"]({{\"history\":{on}}})",
                    n = c.nonce
                );
                let _ = p.send_event(Ev::Eval(tab, js));
            }
        }
        "save_tabs" => match lib().and_then(|mut l| save_tabs(c, &mut l)) {
            Ok((name, paths)) => reply(
                p,
                c,
                tab,
                true,
                format!(
                    "✔ {} tabs → group “{name}”\n{}",
                    paths.len(),
                    paths.join("\n")
                ),
            ),
            Err(e) => reply(p, c, tab, false, e),
        },
        "ask" => match carta::ask_page(&backend(c), &s("q"), &s("title"), &s("url"), &s("text")) {
            Ok(t) => reply(p, c, tab, true, t),
            Err(e) => reply(p, c, tab, false, e),
        },
        "ask_tabs" => {
            let r = lib().and_then(|mut l| {
                let (name, _) = save_tabs(c, &mut l)?;
                let docs = l.resolve_group(&name)?;
                let o = Options {
                    request: &s("q"),
                    ..Default::default()
                };
                let task = match m["task"].as_str() {
                    Some("summarize") => "summarize",
                    Some("extract") => "extract",
                    _ => "synthesize",
                };
                let o = Options {
                    fields: if task == "extract" {
                        s("q")
                            .split(',')
                            .map(|f| f.trim().to_string())
                            .filter(|f| !f.is_empty())
                            .collect()
                    } else {
                        vec![]
                    },
                    ..o
                };
                carta::run(&l, task, &docs, &backend(c), &o, &mut |_| {})
            });
            match r {
                Ok(t) => reply(p, c, tab, true, t),
                Err(e) => reply(p, c, tab, false, e),
            }
        }
        _ => {}
    }
}

#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
const GTK: bool = true;
#[cfg(not(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
)))]
const GTK: bool = false;

fn make_tab(
    window: &Window,
    cfg: &Arc<Cfg>,
    proxy: &EventLoopProxy<Ev>,
    id: usize,
    url: Option<&str>,
) -> WebView {
    let (c2, p2) = (cfg.clone(), proxy.clone());
    let (p3, p4, p5, p6) = (proxy.clone(), proxy.clone(), proxy.clone(), proxy.clone());
    let script = OVERLAY
        .replace("__NONCE__", &cfg.nonce)
        .replace("__TAB__", &id.to_string());
    let mut b = WebViewBuilder::new()
        .with_initialization_script(script)
        .with_new_window_req_handler(move |u, _| {
            let _ = p3.send_event(Ev::NewTab(Some(u)));
            wry::NewWindowResponse::Deny
        })
        .with_document_title_changed_handler(move |t| {
            let _ = p4.send_event(Ev::Title(id, t));
        })
        .with_navigation_handler(move |u| {
            let _ = p5.send_event(Ev::Url(id, u));
            true
        })
        .with_ipc_handler(move |req| {
            let Ok(m) = serde_json::from_str::<Value>(req.body()) else {
                return;
            };
            if m["n"].as_str() != Some(c2.nonce.as_str()) {
                return; // a page script cannot know the per-run nonce
            }
            let (c, p) = (c2.clone(), p2.clone());
            std::thread::spawn(move || handle(&c, &p, &m));
        });
    b = match url {
        Some(u) => b.with_url(normalize(u).unwrap_or_else(|| u.to_string())),
        None => b.with_html(START),
    };
    if let (Some(js), 1) = (cfg.selftest, id) {
        let nonce = cfg.nonce.clone();
        b = b.with_on_page_load_handler(move |ev, _| {
            if matches!(ev, wry::PageLoadEvent::Finished) {
                let _ = p6.send_event(Ev::Eval(
                    id,
                    format!(
                        "setTimeout(()=>window[\"__cc_do_{nonce}\"]{js},{})",
                        if js.contains("ask_tabs") { 3500 } else { 900 }
                    ),
                ));
            }
        });
    }
    if GTK {
        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        {
            use tao::platform::unix::WindowExtUnix;
            use wry::WebViewBuilderExtUnix;
            return b
                .build_gtk(window.default_vbox().expect("gtk vbox"))
                .expect("webview");
        }
    }
    #[cfg(not(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    )))]
    {
        return b
            .with_bounds(full_bounds(window))
            .build_as_child(window)
            .expect("webview");
    }
    #[allow(unreachable_code)]
    {
        unreachable!()
    }
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
)))]
fn full_bounds(window: &Window) -> wry::Rect {
    let size = window.inner_size().to_logical::<f64>(window.scale_factor());
    wry::Rect {
        position: wry::dpi::LogicalPosition::new(0.0, 0.0).into(),
        size: wry::dpi::LogicalSize::new(size.width, size.height).into(),
    }
}

fn push_tabs(cfg: &Cfg, tabs: &[Tab], active: usize) {
    let list: Vec<Value> = tabs
        .iter()
        .map(|t| json!({"id": t.id, "title": t.title, "url": t.url}))
        .collect();
    let st = json!({"tabs": list, "active": active});
    let js = format!(
        "window[\"__cc_tabs_{n}\"]&&window[\"__cc_tabs_{n}\"]({st})",
        n = cfg.nonce
    );
    for t in tabs {
        let _ = t.wv.evaluate_script(&js);
    }
}

fn save_session(cfg: &Cfg, tabs: &[Tab]) {
    if let Ok(h) = state_dir(cfg.home.as_deref()) {
        let urls: Vec<&str> = tabs
            .iter()
            .map(|t| t.url.as_str())
            .filter(|u| u.starts_with("http"))
            .collect();
        let _ = std::fs::write(
            h.join("tabs.json"),
            serde_json::to_string(&urls).unwrap_or_default(),
        );
    }
}

fn show_only(tabs: &[Tab], active: usize) {
    for t in tabs {
        let _ = t.wv.set_visible(t.id == active);
    }
    if let Some(t) = tabs.iter().find(|t| t.id == active) {
        let _ = t.wv.focus();
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let flag = |f: &str| args.iter().any(|a| a == f);
    let val = |f: &str| {
        args.iter()
            .position(|a| a == f)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    if flag("--help") || flag("-h") {
        println!(
            "cartacolibri-browser [URL…] [--home DIR] [--gateway URL] [--mock] [--fresh]\n  --fresh   do not restore the previous tabs\n  --selftest-save | --selftest-ask | --selftest-tabs   load the first URL, save it / ask about it (use with --mock), print the result, exit (used by CI)"
        );
        return;
    }
    let selftest: Option<&'static str> = if flag("--selftest-tabs") {
        Some("('ask_tabs','colibri')")
    } else if flag("--selftest-ask") {
        Some("('ask','ce sunt colibrii?')")
    } else if flag("--selftest-save") {
        Some("('save')")
    } else {
        None
    };
    let (hv, gv) = (val("--home"), val("--gateway"));
    let urls: Vec<String> = args
        .iter()
        .filter(|a| !a.starts_with("--") && Some(*a) != hv.as_ref() && Some(*a) != gv.as_ref())
        .cloned()
        .collect();
    let cfg = Arc::new(Cfg {
        home: hv,
        gateway: gv
            .or_else(|| std::env::var("COLI_URL").ok())
            .unwrap_or_else(|| carta::DEFAULT_URL.into()),
        mock: flag("--mock"),
        nonce: nonce(),
        selftest,
        pages: Mutex::new(BTreeMap::new()),
    });
    let mut start_urls = urls;
    if start_urls.is_empty()
        && !flag("--fresh")
        && selftest.is_none()
        && let Ok(h) = state_dir(cfg.home.as_deref())
        && let Some(v) = std::fs::read_to_string(h.join("tabs.json"))
            .ok()
            .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
    {
        start_urls = v;
    }

    let event_loop = EventLoopBuilder::<Ev>::with_user_event().build();
    let proxy = event_loop.create_proxy();
    let window = WindowBuilder::new()
        .with_title("CartaColibri · web-grab")
        .with_inner_size(tao::dpi::LogicalSize::new(1280.0, 860.0))
        .build(&event_loop)
        .expect("window");

    let mut tabs: Vec<Tab> = vec![];
    let mut next_id = 1usize;
    let mut active = 1usize;
    let first: Vec<Option<String>> = if start_urls.is_empty() {
        vec![None]
    } else {
        start_urls.into_iter().map(Some).collect()
    };
    for u in first {
        let id = next_id;
        next_id += 1;
        let wv = make_tab(&window, &cfg, &proxy, id, u.as_deref());
        tabs.push(Tab {
            id,
            wv,
            title: String::new(),
            url: u.unwrap_or_default(),
        });
        active = id;
    }
    show_only(&tabs, active);

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    event_loop.run(move |event, _, flow| {
        *flow = if selftest.is_some() {
            ControlFlow::WaitUntil(
                std::time::Instant::now() + std::time::Duration::from_millis(250),
            )
        } else {
            ControlFlow::Wait
        };
        match event {
            Event::UserEvent(Ev::Eval(tab, js)) => {
                if selftest.is_some() && js.contains("\"text\":") {
                    let ok = js.contains("\"ok\":true") && (tabs.len() < 2 || js.contains("[S2]"));
                    println!(
                        "selftest {}: {}",
                        if ok { "ok" } else { "FAILED" },
                        js.split("(").last().unwrap_or("")
                    );
                    *flow = ControlFlow::ExitWithCode(if ok { 0 } else { 1 });
                    return;
                }
                if let Some(t) = tabs.iter().find(|t| t.id == tab) {
                    let _ = t.wv.evaluate_script(&js);
                }
            }
            Event::UserEvent(Ev::Navigate(tab, u)) => {
                if let Some(t) = tabs.iter().find(|t| t.id == tab) {
                    let _ = t.wv.load_url(&u);
                }
            }
            Event::UserEvent(Ev::NewTab(u)) => {
                let id = next_id;
                next_id += 1;
                let wv = make_tab(&window, &cfg, &proxy, id, u.as_deref());
                tabs.push(Tab {
                    id,
                    wv,
                    title: String::new(),
                    url: u.unwrap_or_default(),
                });
                active = id;
                show_only(&tabs, active);
                push_tabs(&cfg, &tabs, active);
                save_session(&cfg, &tabs);
            }
            Event::UserEvent(Ev::Close(id)) => {
                if tabs.len() > 1 {
                    if let Some(i) = tabs.iter().position(|t| t.id == id) {
                        tabs.remove(i);
                        cfg.pages.lock().unwrap().remove(&id);
                        if active == id {
                            active = tabs[i.min(tabs.len() - 1)].id;
                        }
                        show_only(&tabs, active);
                        push_tabs(&cfg, &tabs, active);
                        save_session(&cfg, &tabs);
                    }
                } else {
                    *flow = ControlFlow::Exit;
                }
            }
            Event::UserEvent(Ev::Activate(id)) => {
                if tabs.iter().any(|t| t.id == id) {
                    active = id;
                    show_only(&tabs, active);
                    push_tabs(&cfg, &tabs, active);
                }
            }
            Event::UserEvent(Ev::Cycle(back)) => {
                if let Some(i) = tabs.iter().position(|t| t.id == active) {
                    let n = tabs.len();
                    active = tabs[if back { (i + n - 1) % n } else { (i + 1) % n }].id;
                    show_only(&tabs, active);
                    push_tabs(&cfg, &tabs, active);
                }
            }
            Event::UserEvent(Ev::Title(id, title)) => {
                if let Some(t) = tabs.iter_mut().find(|t| t.id == id)
                    && t.title != title
                {
                    t.title = title;
                    push_tabs(&cfg, &tabs, active);
                }
            }
            Event::UserEvent(Ev::Url(id, url)) => {
                if let Some(t) = tabs.iter_mut().find(|t| t.id == id)
                    && t.url != url
                {
                    t.url = url;
                    push_tabs(&cfg, &tabs, active);
                    save_session(&cfg, &tabs);
                }
            }
            Event::NewEvents(StartCause::ResumeTimeReached { .. })
                if selftest.is_some() && std::time::Instant::now() > deadline =>
            {
                eprintln!("selftest FAILED: timeout");
                *flow = ControlFlow::ExitWithCode(2);
            }
            #[cfg(not(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            )))]
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                let b = full_bounds(&window);
                tabs.iter().for_each(|t| {
                    let _ = t.wv.set_bounds(b);
                });
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *flow = ControlFlow::Exit,
            _ => {}
        }
    });
}
