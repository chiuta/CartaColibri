//! `cartacolibri` — command line + local UI server.

mod cli_more;
use cartacolibri::carta::{self, Backend, Options};
use cartacolibri::doctor::{self, Level};
use cartacolibri::library::{Filter, Library};
use cartacolibri::tour::{self, Paint, t};
use cartacolibri::{grab, server};
use clap::{CommandFactory, Parser, Subcommand};
use std::io::{BufRead, IsTerminal, Write};
use std::sync::atomic::AtomicBool;
use std::time::Duration;

#[derive(Parser)]
#[command(
    name = "cartacolibri",
    version,
    about = "CartaColibri — local AI library (RAG-fetch + RAG-CARTA), 100% on your machine",
    after_help = "FIRST TIME? Run:  cartacolibri start\n\n\
EXAMPLES:\n  \
cartacolibri start --folder ~/Documents     index a folder and open the web UI\n  \
cartacolibri search \"invoice 2024\" --ext pdf  find words in your documents\n  \
cartacolibri search budget --json           same, machine-readable\n  \
cartacolibri group from-search taxes \"tax\"   save a search as a group\n  \
cartacolibri summarize --group taxes        summarize a group with citations\n  \
cartacolibri summarize --group taxes --mock offline, no model needed\n  \
cartacolibri doctor                         check that everything works\n  \
cartacolibri completions zsh > _cartacolibri  shell tab-completion\n\n\
LANGUAGE: --lang ro|en|fr|de|es|it|pt|hu (or env CARTA_LANG). Colours: off when piped or NO_COLOR is set.\n\n\
Authors: Alexandru-Ionuț Chiuță and Alexandru Dumbrava. MIT licence."
)]
struct Cli {
    /// state directory (default ~/.cartacolibri or $CARTACOLIBRI_HOME)
    #[arg(long, global = true)]
    home: Option<String>,
    /// also read images and scanned PDF pages with tesseract (CARTACOLIBRI_OCR=1)
    #[arg(long, global = true)]
    ocr: bool,
    /// language of the messages: en ro fr de es it pt hu (env CARTA_LANG)
    #[arg(long, global = true, env = "CARTA_LANG")]
    lang: Option<String>,
    #[command(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(clap::Args)]
struct TaskArgs {
    request: Option<String>,
    #[arg(long)]
    group: Option<String>,
    /// use the files matching this search instead of a group
    #[arg(long)]
    query: Option<String>,
    /// comma-separated fields to extract
    #[arg(long)]
    fields: Option<String>,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    save: bool,
    #[arg(long, env = "COLI_URL", hide_env_values = true, default_value = carta::DEFAULT_URL)]
    url: String,
    #[arg(long)]
    model: Option<String>,
    /// offline extractive fallback (no model)
    #[arg(long)]
    mock: bool,
    /// source characters per model call
    #[arg(long, default_value_t = carta::CTX_CHARS, value_parser = parse_budget)]
    budget: usize,
}

#[derive(Subcommand)]
enum Cmd {
    /// MCP server over stdio: lets other AI apps search, read and ask about your library (read-only)
    Mcp {
        #[arg(long, default_value = "http://127.0.0.1:8000", env = "COLI_URL")]
        url: String,
        /// answer `ask` offline with the deterministic mock instead of the engine
        #[arg(long)]
        mock: bool,
    },
    /// report what is new for a saved search since the last run (optionally on a schedule)
    Digest {
        /// name of a saved search (see `saved list`)
        name: String,
        /// append the report to this Markdown file instead of printing it
        #[arg(long)]
        out: Option<String>,
        /// let the local engine summarize the new documents
        #[arg(long)]
        summarize: bool,
        /// repeat every N minutes (re-indexes the folders first); omit to run once
        #[arg(long, value_parser = parse_positive)]
        every: Option<u64>,
        /// on the first run, report everything that already matches
        #[arg(long)]
        all: bool,
        #[arg(long, env = "COLI_URL", hide_env_values = true, default_value = carta::DEFAULT_URL)]
        url: String,
        #[arg(long)]
        mock: bool,
    },
    /// emails, links, money amounts, dates and phone numbers mentioned in your documents
    Entities {
        #[arg(long, value_parser = ["email", "url", "money", "date", "phone"])]
        kind: Option<String>,
        #[arg(long)]
        group: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// compare two documents sentence by sentence
    Compare {
        a: String,
        b: String,
        #[arg(long)]
        json: bool,
    },
    /// pairs of sentences in different documents that agree except for their numbers or dates
    Conflicts {
        #[arg(long)]
        group: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// group the library by what the documents are about
    Topics {
        #[arg(long)]
        json: bool,
    },
    /// everything the library says about a term: passages per document, dates and amounts
    Dossier {
        term: String,
        #[arg(long)]
        json: bool,
    },
    /// documents modified since a date (YYYY, YYYY-MM or YYYY-MM-DD)
    Changes {
        #[arg(long)]
        since: String,
        #[arg(long)]
        json: bool,
    },
    /// check a text of yours against the library, sentence by sentence ("-" reads stdin)
    Factcheck {
        file: String,
        #[arg(long)]
        json: bool,
    },
    /// guided start: pick a folder, index it, open the web UI
    Start {
        /// folder with your documents (asked interactively when omitted in a terminal)
        #[arg(long)]
        folder: Option<String>,
        #[arg(long, default_value_t = 8765)]
        port: u16,
        #[arg(long, env = "COLI_URL", hide_env_values = true, default_value = carta::DEFAULT_URL)]
        url: String,
        /// do not open the browser
        #[arg(long)]
        no_open: bool,
        /// index only; do not start the UI server
        #[arg(long)]
        no_serve: bool,
    },
    /// check tesseract, PDF tools, gateway, disk and port; prints exact fixes
    Doctor {
        #[arg(long, default_value_t = 8765)]
        port: u16,
        #[arg(long, env = "COLI_URL", hide_env_values = true, default_value = carta::DEFAULT_URL)]
        url: String,
    },
    /// print a shell tab-completion script
    Completions {
        #[arg(value_parser = ["bash", "zsh", "fish", "powershell"])]
        shell: String,
    },
    /// register a library folder (and index it)
    Add {
        folder: String,
    },
    /// forget a folder (files untouched)
    Remove {
        folder: String,
    },
    Folders,
    /// incremental re-index
    Index {
        folder: Option<String>,
    },
    Problems,
    /// keep the index current: re-scan the folders every N seconds
    Watch {
        #[arg(long, default_value_t = 30, value_parser = parse_positive)]
        interval: u64,
    },
    List {
        #[arg(long)]
        group: Option<String>,
        /// machine-readable output
        #[arg(long)]
        json: bool,
    },
    Search {
        query: String,
        /// comma-separated, e.g. pdf,txt
        #[arg(long)]
        ext: Option<String>,
        #[arg(long, value_parser = parse_kb)]
        min_kb: Option<f64>,
        #[arg(long, value_parser = parse_kb)]
        max_kb: Option<f64>,
        #[arg(long)]
        path: Option<String>,
        #[arg(long)]
        group: Option<String>,
        #[arg(long, default_value_t = 20)]
        limit: usize,
        /// machine-readable output
        #[arg(long)]
        json: bool,
    },
    #[command(subcommand)]
    Group(GroupCmd),
    #[command(flatten)]
    More(cli_more::More),
    Summarize(TaskArgs),
    Extract(TaskArgs),
    Synthesize(TaskArgs),
    /// web-grab: fetch a web page into the library
    Grab {
        url: String,
    },
    /// watched web pages: report (and optionally refresh) what changed since you saved them
    #[command(subcommand)]
    Webwatch(WwCmd),
    /// searchable browsing history of the web-grab browser (opt-in, local)
    History {
        #[arg(value_parser = ["on", "off", "clear", "status"])]
        action: String,
    },
    /// print the web-grab bookmarklet (drag it to your browser's bookmarks bar; works in any browser)
    Bookmarklet {
        #[arg(long, default_value_t = 8765)]
        port: u16,
    },
    /// start the local UI (http://127.0.0.1:PORT)
    Serve {
        #[arg(long, default_value_t = 8765)]
        port: u16,
        #[arg(long, env = "COLI_URL", hide_env_values = true, default_value = carta::DEFAULT_URL)]
        url: String,
    },
}

#[derive(Subcommand)]
enum WwCmd {
    Add {
        url: String,
    },
    List,
    Rm {
        url: String,
    },
    /// check every watched page once
    Check {
        /// also update the saved copy of changed pages
        #[arg(long)]
        refresh: bool,
    },
    /// check forever, every N minutes
    Run {
        #[arg(long, default_value_t = 60, value_parser = parse_positive)]
        minutes: u64,
        #[arg(long)]
        refresh: bool,
    },
}

#[derive(Subcommand)]
enum GroupCmd {
    Set {
        name: String,
        members: Vec<String>,
        #[arg(short, long)]
        description: Option<String>,
        #[arg(long)]
        add: bool,
    },
    List,
    Show {
        name: String,
    },
    Rm {
        name: String,
    },
    /// snapshot search results as a group
    FromSearch {
        name: String,
        query: String,
        #[arg(long)]
        ext: Option<String>,
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
}

fn parse_budget(s: &str) -> Result<usize, String> {
    match s.trim().parse::<usize>() {
        Ok(n) if n >= 100 => Ok(n),
        Ok(_) => Err("must be at least 100 characters".into()),
        Err(_) => Err(format!("`{s}` is not a whole number")),
    }
}

fn parse_positive(s: &str) -> Result<u64, String> {
    match s.trim().parse::<u64>() {
        Ok(n) if (1..=31_536_000).contains(&n) => Ok(n),
        _ => Err(format!("`{s}` must be a whole number from 1 to 31536000")),
    }
}

fn parse_kb(s: &str) -> Result<f64, String> {
    match s.trim().parse::<f64>() {
        Ok(n) if n.is_finite() && n >= 0.0 => Ok(n),
        _ => Err(format!("`{s}` must be a number of KB, 0 or more")),
    }
}

fn csv(s: &Option<String>) -> Vec<String> {
    s.as_deref()
        .map(|v| {
            v.split(',')
                .map(|x| x.trim().to_string())
                .filter(|x| !x.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn main() {
    // `cartacolibri list | head` closes stdout early: exit quietly instead of a panic backtrace
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let msg = info
            .payload()
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| info.payload().downcast_ref::<&str>().copied())
            .unwrap_or("");
        if msg.contains("failed printing to std") {
            std::process::exit(0);
        }
        default_hook(info)
    }));
    let c = Cli::parse();
    let lang = tour::resolve(c.lang.as_deref());
    if let Err(m) = run(c, lang) {
        tour::print_error(lang, &m);
        std::process::exit(1);
    }
}

fn opener(url: &str) {
    let (cmd, args): (&str, Vec<&str>) = if cfg!(target_os = "macos") {
        ("open", vec![url])
    } else if cfg!(target_os = "windows") {
        ("cmd", vec!["/C", "start", "", url])
    } else {
        ("xdg-open", vec![url])
    };
    let _ = std::process::Command::new(cmd)
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}

fn welcome(lang: usize, home: Option<&str>) -> Result<(), String> {
    let p = Paint::stdout();
    let lib = Library::open(home, false)?;
    let roots = lib.roots();
    println!();
    if roots.is_empty() {
        println!("  {}", p.bold(&t(lang, "hello1", &[])));
        println!("  {}", t(lang, "hello2", &[]));
        println!("  {}", t(lang, "hello3", &[]));
        println!();
        println!("      {}", p.cyan("cartacolibri start"));
        println!();
        println!("  {}", p.dim("cartacolibri --help"));
    } else {
        let docs = lib.documents(None)?.len();
        println!(
            "  {}",
            p.bold(&t(
                lang,
                "status",
                &[&docs.to_string(), &roots.len().to_string()]
            ))
        );
        println!("  {}", t(lang, "next", &[]));
        println!();
        for (cmd, key) in [
            ("cartacolibri search \"your words\"", "do_search"),
            ("cartacolibri start", "do_ui"),
            ("cartacolibri doctor", "do_doctor"),
        ] {
            println!(
                "      {}   {}",
                p.cyan(&format!("{cmd:<34}")),
                p.dim(&t(lang, key, &[]))
            );
        }
        println!();
    }
    Ok(())
}

fn ask_folder(lang: usize, default: Option<&str>) -> Result<String, String> {
    let p = Paint::stdout();
    match default {
        Some(d) => print!("{} [{d}] ", p.bold(&t(lang, "ask_folder", &[]))),
        None => print!("{} ", p.bold(&t(lang, "ask_folder", &[]))),
    }
    let _ = std::io::stdout().flush();
    let mut l = String::new();
    std::io::stdin()
        .lock()
        .read_line(&mut l)
        .map_err(|e| e.to_string())?;
    let l = l.trim().trim_matches(|c| c == '"' || c == '\'').to_string();
    if l.is_empty() {
        default
            .map(String::from)
            .ok_or_else(|| t(lang, "need_folder", &[]))
    } else {
        Ok(l)
    }
}

fn start(
    lang: usize,
    home: Option<&str>,
    ocr: bool,
    (folder, port, url, no_open, no_serve): (Option<String>, u16, String, bool, bool),
) -> Result<(), String> {
    let p = Paint::stdout();
    let mut lib = Library::open(home, ocr)?;
    let existing = lib.roots();
    let folder = match folder {
        Some(f) => Some(f),
        None if std::io::stdin().is_terminal() => {
            Some(ask_folder(lang, existing.first().map(String::as_str))?)
        }
        None if existing.is_empty() => {
            return Err(format!(
                "{} (cartacolibri start --folder ~/Documents)",
                t(lang, "need_folder", &[])
            ));
        }
        None => None,
    };
    let list: Option<Vec<String>> = folder.map(|f| vec![f]);
    println!("{}", t(lang, "indexing", &[]));
    if let Some(l) = &list {
        lib.add_root(&l[0])?;
        cartacolibri::organize::snapshot_roots(&lib);
    }
    let st = lib.update(list.as_deref(), &mut |m| eprintln!("  {}", p.dim(m)))?;
    println!(
        "{}",
        p.green(&t(
            lang,
            "indexed",
            &[
                &(st.added).to_string(),
                &st.updated.to_string(),
                &st.unchanged.to_string(),
                &st.failed.to_string()
            ]
        ))
    );
    println!();
    println!("{}", t(lang, "search_how", &[]));
    println!("    {}", p.cyan("cartacolibri search \"your words\""));
    let ui = format!("http://127.0.0.1:{port}/");
    println!("{}", t(lang, "ui_at", &[&p.bold(&ui)]));
    if no_serve {
        return Ok(());
    }
    if std::net::TcpListener::bind(("127.0.0.1", port)).is_err() {
        return Err(format!("cannot listen on 127.0.0.1:{port}: address in use"));
    }
    println!("{}", p.dim(&t(lang, "stop_hint", &[])));
    if !no_open {
        let u = ui.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(400));
            opener(&u)
        });
    }
    drop(lib);
    server::serve(&format!("127.0.0.1:{port}"), home.map(String::from), url)
}

fn doctor_cmd(lang: usize, home: Option<&str>, url: &str, port: u16) -> Result<(), String> {
    let p = Paint::stdout();
    println!("{}\n", p.bold(&t(lang, "doctor_title", &[])));
    let checks = doctor::run(home, url, port);
    let (mut warns, mut fails) = (0, 0);
    for c in &checks {
        let tag = match c.level {
            Level::Ok => p.green("OK  "),
            Level::Warn => {
                warns += 1;
                p.yellow("WARN")
            }
            Level::Fail => {
                fails += 1;
                p.red("FAIL")
            }
        };
        println!("{tag}  {:<9} {}", c.name, c.detail);
        if let Some(f) = &c.fix {
            println!("      {} {f}", p.bold("fix:"));
        }
    }
    println!();
    if fails > 0 {
        println!("{}", p.red(&t(lang, "doctor_bad", &[&fails.to_string()])));
        std::process::exit(1);
    } else if warns > 0 {
        println!("{}", t(lang, "doctor_warns", &[&warns.to_string()]));
    } else {
        println!("{}", p.green(&t(lang, "doctor_good", &[])));
    }
    Ok(())
}

fn run(c: Cli, lang: usize) -> Result<(), String> {
    let home = c.home.as_deref();
    let Some(cmd) = c.cmd else {
        return welcome(lang, home);
    };
    match &cmd {
        Cmd::Completions { shell } => {
            use clap_complete::Shell;
            let sh = match shell.as_str() {
                "bash" => Shell::Bash,
                "zsh" => Shell::Zsh,
                "fish" => Shell::Fish,
                _ => Shell::PowerShell,
            };
            clap_complete::generate(
                sh,
                &mut Cli::command(),
                "cartacolibri",
                &mut std::io::stdout(),
            );
            return Ok(());
        }
        Cmd::Doctor { port, url } => return doctor_cmd(lang, home, url, *port),
        Cmd::More(cli_more::More::Restore { zip }) => return cli_more::restore(home, zip),
        Cmd::More(cli_more::More::Verify { repair }) => return cli_more::verify_cmd(home, *repair),
        Cmd::Start {
            folder,
            port,
            url,
            no_open,
            no_serve,
        } => {
            return start(
                lang,
                home,
                c.ocr,
                (folder.clone(), *port, url.clone(), *no_open, *no_serve),
            );
        }
        _ => {}
    }
    if let Cmd::Serve { port, url } = &cmd {
        if let Ok(n) = std::env::var("CARTACOLIBRI_WATCH")
            .unwrap_or_default()
            .parse::<u64>()
            && n > 0
        {
            let h = home.map(String::from);
            std::thread::spawn(move || {
                if let Ok(mut lib) = Library::open(h.as_deref(), false) {
                    lib.watch(Duration::from_secs(n), &AtomicBool::new(false), &mut |m| {
                        eprintln!("watch: {m}")
                    });
                }
            });
        }
        let addr = format!("127.0.0.1:{port}");
        let h = Library::open(home, false)?.home;
        println!(
            "CartaColibri UI: http://{addr}/   (gateway: {})",
            carta::redact_url(url)
        );
        println!(
            "web-grab bookmarklet token is in {}",
            h.join("clip.token").display()
        );
        return server::serve(&addr, home.map(String::from), url.clone());
    }
    let mut lib = Library::open(home, c.ocr)?;
    match cmd {
        Cmd::Start { .. } | Cmd::Doctor { .. } | Cmd::Completions { .. } => unreachable!(),
        Cmd::Mcp { url, mock } => {
            let be = if mock {
                carta::Backend::mock()
            } else {
                carta::Backend::gateway(&url, None)
            };
            cartacolibri::mcp::serve_stdio(&lib, &be).map_err(|e| e.to_string())?;
        }
        Cmd::More(m) => cli_more::run(&mut lib, m)?,
        Cmd::History { action } => match action.as_str() {
            "on" => {
                grab::set_history(&lib.home, true);
                println!(
                    "history on: pages you visit in cartacolibri-browser are kept locally and become searchable"
                );
            }
            "off" => {
                grab::set_history(&lib.home, false);
                println!("history off (existing entries stay until `history clear`)");
            }
            "clear" => println!(
                "history cleared ({} documents dropped from the index)",
                grab::history_clear(&lib)?
            ),
            _ => println!(
                "history is {}",
                if grab::history_enabled(&lib.home) {
                    "on"
                } else {
                    "off"
                }
            ),
        },
        Cmd::Webwatch(w) => match w {
            WwCmd::Add { url } => {
                if !(url.starts_with("http://") || url.starts_with("https://")) {
                    return Err(format!(
                        "`{url}` is not a web address (http:// or https://)"
                    ));
                }
                let mut l = grab::watch_list(&lib.home);
                if !l.contains(&url) {
                    l.push(url.clone());
                }
                grab::watch_set(&lib.home, l);
                println!("watching {url}");
            }
            WwCmd::List => grab::watch_list(&lib.home)
                .iter()
                .for_each(|u| println!("{u}")),
            WwCmd::Rm { url } => {
                let l: Vec<String> = grab::watch_list(&lib.home)
                    .into_iter()
                    .filter(|u| *u != url)
                    .collect();
                grab::watch_set(&lib.home, l);
                println!("no longer watching {url}");
            }
            WwCmd::Check { refresh } => print_watch(&grab::watch_check(&mut lib, refresh)),
            WwCmd::Run { minutes, refresh } => loop {
                print_watch(&grab::watch_check(&mut lib, refresh));
                std::thread::sleep(Duration::from_secs(minutes.saturating_mul(60)));
            },
        },
        Cmd::Bookmarklet { port } => {
            let tok = server::token(&lib.home);
            println!(
                "javascript:(async()=>{{try{{const s=String(getSelection());const r=await fetch('http://127.0.0.1:{port}/carta/clip?token={tok}',{{method:'POST',headers:{{'Content-Type':'text/plain'}},body:JSON.stringify({{url:location.href,title:document.title,text:s||document.body.innerText}})}});const j=await r.json();alert(r.ok?'Saved to CartaColibri ('+j.chars+' characters)':'CartaColibri: '+j.error)}}catch(e){{alert('CartaColibri is not running on 127.0.0.1:{port}: '+e)}}}})()"
            );
        }
        Cmd::Add { folder } => {
            println!("registered {}", lib.add_root(&folder)?);
            cartacolibri::organize::snapshot_roots(&lib);
            println!(
                "{:?}",
                lib.update(Some(&[folder]), &mut |m| eprintln!("{m}"))?
            );
        }
        Cmd::Remove { folder } => {
            println!(
                "forgot {folder} ({} documents dropped from the index; files untouched)",
                lib.remove_root(&folder)?
            );
            cartacolibri::organize::snapshot_roots(&lib);
        }
        Cmd::Folders => lib.roots().iter().for_each(|r| println!("{r}")),
        Cmd::Index { folder } => {
            let r = folder.map(|f| vec![f]);
            println!("{:?}", lib.update(r.as_deref(), &mut |m| eprintln!("{m}"))?)
        }
        Cmd::Problems => lib.problems().iter().for_each(|p| {
            println!(
                "{}\n    {}",
                p["path"].as_str().unwrap_or(""),
                p["reason"].as_str().unwrap_or("")
            )
        }),
        Cmd::Digest {
            name,
            out,
            summarize,
            every,
            all,
            url,
            mock,
        } => {
            let be = if mock {
                carta::Backend::mock()
            } else {
                carta::Backend::gateway(&url, None)
            };
            loop {
                if every.is_some() {
                    let _ = lib.update(None, &mut |_| {});
                }
                match cartacolibri::digest::run_once(&lib, &name, summarize.then_some(&be), all)? {
                    Some(md) => match &out {
                        Some(f) => {
                            use std::io::Write;
                            let mut fh = std::fs::OpenOptions::new()
                                .create(true)
                                .append(true)
                                .open(f)
                                .map_err(|e| e.to_string())?;
                            writeln!(fh, "{md}").map_err(|e| e.to_string())?;
                            println!("{name}: new documents appended to {f}");
                        }
                        None => println!("{md}"),
                    },
                    None => println!("{name}: nothing new"),
                }
                match every {
                    Some(m) => std::thread::sleep(Duration::from_secs(m * 60)),
                    None => break,
                }
            }
        }
        Cmd::Entities { kind, group, json } => {
            let docs: Vec<(String, String)> = lib
                .documents(group.as_deref())?
                .into_iter()
                .map(|d| (d.path, lib.text_of(d.id)))
                .collect();
            let idx = cartacolibri::insight::entity_index(&docs, kind.as_deref());
            if json {
                let v: Vec<_> = idx
                    .iter()
                    .map(|((k, v), p)| serde_json::json!({"kind": k, "value": v, "docs": p}))
                    .collect();
                println!("{}", serde_json::to_string_pretty(&v).unwrap());
            } else if idx.is_empty() {
                println!("no entities found");
            }
            if !json {
                for ((k, v), p) in &idx {
                    println!(
                        "{k:<6} {v}   ({} doc{})",
                        p.len(),
                        if p.len() == 1 { "" } else { "s" }
                    );
                }
            }
        }
        Cmd::Compare { a, b, json } => {
            let find = |name: &str| -> Result<(String, String), String> {
                let n = name.to_lowercase();
                let docs = lib.documents(None)?;
                let d = docs
                    .iter()
                    .find(|d| d.path.to_lowercase() == n)
                    .or_else(|| docs.iter().find(|d| d.path.to_lowercase().ends_with(&n)))
                    .ok_or(format!("no document matching '{name}' in the library"))?;
                Ok((d.path.clone(), lib.text_of(d.id)))
            };
            let ((pa, ta), (pb, tb)) = (find(&a)?, find(&b)?);
            let c = cartacolibri::insight::compare(&ta, &tb);
            if json {
                println!("{}", serde_json::to_string_pretty(&c).unwrap());
            } else {
                println!(
                    "{}% alike — {} shared sentence(s)\n",
                    c.similarity, c.shared
                );
                c.only_in_a
                    .iter()
                    .for_each(|s| println!("- only in {pa}: {s}"));
                c.only_in_b
                    .iter()
                    .for_each(|s| println!("+ only in {pb}: {s}"));
            }
        }
        Cmd::Conflicts { group, json } => {
            let docs = cartacolibri::insight::library_texts(&lib, group.as_deref())?;
            let c = cartacolibri::insight::conflicts(&docs);
            if json {
                println!("{}", serde_json::to_string_pretty(&c).unwrap());
            } else if c.is_empty() {
                println!("no conflicting numbers found");
            }
            if !json {
                for x in c {
                    println!(
                        "{} ({}) vs {} ({})\n  A: {}\n  B: {}\n",
                        x.a_path,
                        x.a_only.join(", "),
                        x.b_path,
                        x.b_only.join(", "),
                        x.a_sentence,
                        x.b_sentence
                    );
                }
            }
        }
        Cmd::Topics { json } => {
            let docs = cartacolibri::insight::library_texts(&lib, None)?;
            let t = cartacolibri::insight::topics(&docs);
            if json {
                println!("{}", serde_json::to_string_pretty(&t).unwrap());
            } else if t.is_empty() {
                println!("no groups of related documents yet");
            }
            if !json {
                for x in t {
                    println!("{} ({} documents)", x.terms.join(", "), x.docs.len());
                    x.docs.iter().for_each(|d| println!("    {d}"));
                }
            }
        }
        Cmd::Dossier { term, json } => {
            let d = cartacolibri::insight::dossier(&lib, &term)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&d).unwrap());
            } else {
                for x in &d.docs {
                    println!("{}", x.path);
                    x.passages.iter().for_each(|p| println!("    - {p}"));
                }
                if !d.dates.is_empty() {
                    println!("dates: {}", d.dates.join(", "));
                }
                if !d.money.is_empty() {
                    println!("amounts: {}", d.money.join(", "));
                }
                if d.docs.is_empty() {
                    println!("nothing about '{}' in the library", d.term);
                }
            }
        }
        Cmd::Changes { since, json } => {
            let t = cartacolibri::library::parse_date(&since)
                .ok_or("give --since as YYYY, YYYY-MM or YYYY-MM-DD")?;
            let c = cartacolibri::insight::changes(&lib, t, 200)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&c).unwrap());
            } else {
                for d in &c {
                    println!("{:>7} KB  {}", d.size / 1024, d.path);
                }
                if c.is_empty() {
                    println!("nothing modified since {since}");
                }
            }
        }
        Cmd::Factcheck { file, json } => {
            let text = if file == "-" {
                let mut b = String::new();
                std::io::Read::read_to_string(&mut std::io::stdin(), &mut b)
                    .map_err(|e| e.to_string())?;
                b
            } else {
                std::fs::read_to_string(&file).map_err(|e| format!("cannot read {file}: {e}"))?
            };
            let c = cartacolibri::insight::factcheck(&lib, &text)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&c).unwrap());
            } else {
                for x in c {
                    println!("[{}] {}", x.status, x.sentence);
                    if let (Some(s), Some(e)) = (&x.source, &x.evidence) {
                        println!("      {s}: {e}");
                    }
                }
            }
        }
        Cmd::Watch { interval } => {
            println!(
                "watching {} folder(s) every {interval}s — Ctrl+C to stop",
                lib.roots().len()
            );
            lib.watch(
                Duration::from_secs(interval),
                &AtomicBool::new(false),
                &mut |m| println!("{m}"),
            );
        }
        Cmd::List { group, json } => {
            let docs = lib.documents(group.as_deref())?;
            if json {
                println!("{}", serde_json::to_string_pretty(&docs).unwrap());
            } else if docs.is_empty() {
                println!("{}", t(lang, "empty_lib", &[]));
                println!("{} cartacolibri start", t(lang, "try", &[]));
            }
            if !json {
                for d in docs {
                    println!(
                        "{:>7} KB  {:<6} {}",
                        d.size / 1024,
                        tour::plain(&d.ext),
                        tour::plain(&d.path)
                    )
                }
            }
        }
        Cmd::Search {
            query,
            ext,
            min_kb,
            max_kb,
            path,
            group,
            limit,
            json,
        } => {
            let f = Filter {
                ext: csv(&ext),
                min_size: min_kb.map(|k| (k * 1024.0) as i64),
                max_size: max_kb.map(|k| (k * 1024.0) as i64),
                path_like: path,
                group,
                limit: Some(limit),
                ..Default::default()
            };
            let hits = lib.search(&query, &f)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&hits).unwrap());
                return Ok(());
            }
            for h in &hits {
                println!(
                    "{:>7}  {} {}\n         {}",
                    h.score,
                    tour::plain(&h.path),
                    tour::plain(&h.loc),
                    tour::plain(&h.snippet)
                );
            }
            if hits.is_empty() {
                println!("{}", t(lang, "no_results", &[&query]));
                if lib.documents(None)?.is_empty() {
                    println!("{}", t(lang, "empty_lib", &[]));
                    println!("{} {}", t(lang, "try", &[]), t(lang, "no_docs_hint", &[]));
                } else {
                    println!("{} {}", t(lang, "try", &[]), t(lang, "loosen_hint", &[]));
                }
            }
        }
        Cmd::Group(g) => match g {
            GroupCmd::Set {
                name,
                members,
                description,
                add,
            } => println!(
                "{:?}",
                lib.group_set(&name, &members, description.as_deref(), add)?
            ),
            GroupCmd::List => {
                for (n, g) in lib.groups() {
                    println!(
                        "{n}: {} ({} docs)",
                        g.description,
                        lib.resolve_group(&n)?.len()
                    )
                }
            }
            GroupCmd::Show { name } => lib
                .resolve_group(&name)?
                .iter()
                .for_each(|d| println!("{}", d.path)),
            GroupCmd::Rm { name } => {
                lib.group_remove(&name, &[])?;
                println!("removed {name}")
            }
            GroupCmd::FromSearch {
                name,
                query,
                ext,
                limit,
            } => {
                let hits = lib.search(
                    &query,
                    &Filter {
                        ext: csv(&ext),
                        limit: Some(limit),
                        ..Default::default()
                    },
                )?;
                let paths: Vec<String> = hits.iter().map(|h| h.path.clone()).collect();
                println!(
                    "{:?}",
                    lib.group_set(&name, &paths, Some(&format!("search: {query}")), false)?
                )
            }
        },
        Cmd::Summarize(a) => task(&lib, "summarize", a)?,
        Cmd::Extract(a) => task(&lib, "extract", a)?,
        Cmd::Synthesize(a) => task(&lib, "synthesize", a)?,
        Cmd::Grab { url } => println!(
            "{}",
            serde_json::to_string_pretty(&grab::grab(&mut lib, &url)?).unwrap()
        ),
        Cmd::Serve { .. } => unreachable!(),
    }
    Ok(())
}

fn task(lib: &Library, name: &str, a: TaskArgs) -> Result<(), String> {
    let docs = match (&a.group, &a.query) {
        (Some(g), _) => lib.resolve_group(g)?,
        (None, Some(q)) => {
            let ids: Vec<i64> = lib
                .search(
                    q,
                    &Filter {
                        limit: Some(50),
                        ..Default::default()
                    },
                )?
                .iter()
                .map(|h| h.id)
                .collect();
            lib.docs_by_id(&ids)
        }
        _ => return Err("give --group NAME (or --query TEXT)".into()),
    };
    let be = if a.mock {
        Backend::mock()
    } else {
        Backend::gateway(&a.url, a.model.clone())
    };
    let request = a.request.clone().unwrap_or_default();
    let o = Options {
        request: &request,
        fields: csv(&a.fields),
        json: a.json,
        budget: a.budget,
    };
    let text = carta::run(lib, name, &docs, &be, &o, &mut |m| eprintln!("· {m}"))?;
    println!("{}", tour::plain(&text));
    if a.save {
        println!(
            "saved: {}",
            carta::save(
                &lib.home,
                &text,
                name,
                a.group.as_deref().unwrap_or("selection"),
                &request
            )?
        );
    }
    Ok(())
}

fn print_watch(res: &[serde_json::Value]) {
    for r in res {
        let url = r["url"].as_str().unwrap_or("");
        match r["state"].as_str().unwrap_or("") {
            "changed" => {
                println!(
                    "CHANGED  {url}{}",
                    if r["refreshed"] == true {
                        "  (copy refreshed)"
                    } else {
                        ""
                    }
                );
                for l in r["diff"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|l| l.as_str())
                {
                    println!("    {l}");
                }
            }
            "same" => println!("same     {url}"),
            "not-saved" => println!(
                "not saved yet  {url}  (save it with `cartacolibri grab` or 💾 in the browser)"
            ),
            _ => println!("error    {url}  {}", r["error"].as_str().unwrap_or("")),
        }
    }
}
