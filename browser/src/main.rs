//! web-grab browser — a small Rust browser shell on the system webview (WebKitGTK / WebView2 / WKWebView).
//! A slim bar on every page: back / forward / reload, address bar (type words to search your library instead),
//! 💾 save the page or selection into the CartaColibri library (address kept as the source), ✨ ask your local
//! Colibrì about the page. Nothing leaves your machine except the page fetches you ask for.
//!
//! Clean-room, MIT. It reuses no Falkon (GPL-3.0) or other browser code.

use cartacolibri::carta::{self, Backend};
use cartacolibri::grab;
use cartacolibri::library::{Filter, Library};
use serde_json::{Value, json};
use std::sync::Arc;
use tao::event::{Event, StartCause, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy};
use tao::window::WindowBuilder;
use wry::WebViewBuilder;

const OVERLAY: &str = include_str!("overlay.js");
const START: &str = r#"<!doctype html><meta charset=utf-8><title>CartaColibri web-grab</title>
<body style="font:18px system-ui;background:#0f1115;color:#eef1f6;display:grid;place-content:center;height:80vh;text-align:center">
<h1 style="color:#5fd7e6">CartaColibri · web-grab</h1>
<p>Type an address in the bar, or words to search your library.<br>Open a page, then press <b>💾</b> to keep it or <b>✨</b> to ask about it.</p></body>"#;

enum Ev {
    Eval(String),
    Navigate(String),
}

struct Cfg {
    home: Option<String>,
    gateway: String,
    mock: bool,
    nonce: String,
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

fn reply(p: &EventLoopProxy<Ev>, c: &Cfg, ok: bool, text: String) {
    let msg = json!({"ok": ok, "text": text});
    let js = format!(
        "window[\"__cc_{n}\"]&&window[\"__cc_{n}\"]({msg})",
        n = c.nonce
    );
    let _ = p.send_event(Ev::Eval(js));
}

fn handle(c: &Cfg, p: &EventLoopProxy<Ev>, m: &Value) {
    let s = |k: &str| m[k].as_str().unwrap_or("").to_string();
    match m["cmd"].as_str().unwrap_or("") {
        "go" => match normalize(&s("q")) {
            Some(u) => {
                let _ = p.send_event(Ev::Navigate(u));
            }
            None => {
                let out = Library::open(c.home.as_deref(), false)
                    .and_then(|lib| {
                        lib.search(
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
                    Ok(t) => reply(p, c, true, t),
                    Err(e) => reply(p, c, false, e),
                }
            }
        },
        "save" => {
            let r = Library::open(c.home.as_deref(), false)
                .and_then(|mut lib| grab::save(&mut lib, &s("url"), &s("title"), &s("text")));
            match r {
                Ok(v) => reply(
                    p,
                    c,
                    true,
                    format!(
                        "✔ {} characters → {}",
                        v["chars"],
                        v["saved"].as_str().unwrap_or("")
                    ),
                ),
                Err(e) => reply(p, c, false, e),
            }
        }
        "ask" => {
            let be = if c.mock {
                Backend::mock()
            } else {
                Backend::gateway(&c.gateway, None)
            };
            match carta::ask_page(&be, &s("q"), &s("title"), &s("url"), &s("text")) {
                Ok(t) => reply(p, c, true, t),
                Err(e) => reply(p, c, false, e),
            }
        }
        _ => {}
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
            "cartacolibri-browser [URL] [--home DIR] [--gateway URL] [--mock]\n  --selftest-save | --selftest-ask  load URL, save it / ask about it (use with --mock), print the result and exit (used by CI)"
        );
        return;
    }
    let cfg = Arc::new(Cfg {
        home: val("--home"),
        gateway: val("--gateway")
            .or_else(|| std::env::var("COLI_URL").ok())
            .unwrap_or_else(|| carta::DEFAULT_URL.into()),
        mock: flag("--mock"),
        nonce: nonce(),
    });
    let selftest = flag("--selftest-save") || flag("--selftest-ask");
    let selftest_js = if flag("--selftest-ask") {
        "('ask','ce sunt colibrii?')"
    } else {
        "('save')"
    };
    let start = args
        .iter()
        .find(|a| {
            !a.starts_with("--")
                && Some(a.as_str()) != val("--home").as_deref()
                && Some(a.as_str()) != val("--gateway").as_deref()
        })
        .cloned();

    let event_loop = EventLoopBuilder::<Ev>::with_user_event().build();
    let proxy = event_loop.create_proxy();
    let window = WindowBuilder::new()
        .with_title("CartaColibri · web-grab")
        .with_inner_size(tao::dpi::LogicalSize::new(1280.0, 860.0))
        .build(&event_loop)
        .expect("window");

    let (c2, p2) = (cfg.clone(), proxy.clone());
    let p3 = proxy.clone();
    let mut b = WebViewBuilder::new()
        .with_initialization_script(OVERLAY.replace("__NONCE__", &cfg.nonce))
        .with_new_window_req_handler(move |u, _| {
            let _ = p3.send_event(Ev::Navigate(u));
            wry::NewWindowResponse::Deny
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
    b = match &start {
        Some(u) => b.with_url(normalize(u).unwrap_or_else(|| u.clone())),
        None => b.with_html(START),
    };
    if selftest {
        let (c, p) = (cfg.clone(), proxy.clone());
        b = b.with_on_page_load_handler(move |ev, _| {
            if matches!(ev, wry::PageLoadEvent::Finished) {
                let js = format!(
                    "setTimeout(()=>window[\"__cc_do_{}\"]{selftest_js},300)",
                    c.nonce
                );
                let _ = p.send_event(Ev::Eval(js));
            }
        });
    }
    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    let webview = {
        use tao::platform::unix::WindowExtUnix;
        use wry::WebViewBuilderExtUnix;
        b.build_gtk(window.default_vbox().expect("gtk vbox"))
            .expect("webview")
    };
    #[cfg(not(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    )))]
    let webview = b.build(&window).expect("webview");

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(40);
    event_loop.run(move |event, _, flow| {
        *flow = if selftest {
            ControlFlow::WaitUntil(
                std::time::Instant::now() + std::time::Duration::from_millis(250),
            )
        } else {
            ControlFlow::Wait
        };
        match event {
            Event::UserEvent(Ev::Eval(js)) => {
                if selftest && js.contains("\"text\"") {
                    // the reply of the self-test save
                    let ok = js.contains("\"ok\":true");
                    println!(
                        "selftest {}: {}",
                        if ok { "ok" } else { "FAILED" },
                        js.split("(").last().unwrap_or("")
                    );
                    *flow = ControlFlow::ExitWithCode(if ok { 0 } else { 1 });
                    return;
                }
                let _ = webview.evaluate_script(&js);
            }
            Event::UserEvent(Ev::Navigate(u)) => {
                let _ = webview.load_url(&u);
            }
            Event::NewEvents(StartCause::ResumeTimeReached { .. })
                if selftest && std::time::Instant::now() > deadline =>
            {
                eprintln!("selftest FAILED: timeout");
                *flow = ControlFlow::ExitWithCode(2);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *flow = ControlFlow::Exit,
            _ => {}
        }
    });
}
