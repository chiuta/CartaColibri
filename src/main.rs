//! `cartacolibri` — command line + local UI server.

use cartacolibri::carta::{self, Backend, Options};
use cartacolibri::library::{Filter, Library};
use cartacolibri::{grab, server};
use clap::{Parser, Subcommand};
use std::sync::atomic::AtomicBool;
use std::time::Duration;

#[derive(Parser)]
#[command(
    name = "cartacolibri",
    version,
    about = "CartaColibri — local AI library (RAG-fetch + RAG-CARTA), 100% on your machine"
)]
struct Cli {
    /// state directory (default ~/.cartacolibri or $CARTACOLIBRI_HOME)
    #[arg(long, global = true)]
    home: Option<String>,
    /// also read images and scanned PDF pages with tesseract (CARTACOLIBRI_OCR=1)
    #[arg(long, global = true)]
    ocr: bool,
    #[command(subcommand)]
    cmd: Cmd,
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
    #[arg(long, env = "COLI_URL", default_value = carta::DEFAULT_URL)]
    url: String,
    #[arg(long)]
    model: Option<String>,
    /// offline extractive fallback (no model)
    #[arg(long)]
    mock: bool,
    /// source characters per model call
    #[arg(long, default_value_t = carta::CTX_CHARS)]
    budget: usize,
}

#[derive(Subcommand)]
enum Cmd {
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
        #[arg(long, default_value_t = 30)]
        interval: u64,
    },
    List {
        #[arg(long)]
        group: Option<String>,
    },
    Search {
        query: String,
        /// comma-separated, e.g. pdf,txt
        #[arg(long)]
        ext: Option<String>,
        #[arg(long)]
        min_kb: Option<f64>,
        #[arg(long)]
        max_kb: Option<f64>,
        #[arg(long)]
        path: Option<String>,
        #[arg(long)]
        group: Option<String>,
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    #[command(subcommand)]
    Group(GroupCmd),
    Summarize(TaskArgs),
    Extract(TaskArgs),
    Synthesize(TaskArgs),
    /// web-grab: fetch a web page into the library
    Grab {
        url: String,
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
        #[arg(long, env = "COLI_URL", default_value = carta::DEFAULT_URL)]
        url: String,
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
    if let Err(m) = run(Cli::parse()) {
        eprintln!("error: {m}");
        std::process::exit(1);
    }
}

fn run(c: Cli) -> Result<(), String> {
    let home = c.home.as_deref();
    if let Cmd::Serve { port, url } = &c.cmd {
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
        println!("CartaColibri UI: http://{addr}/   (gateway: {url})");
        println!(
            "web-grab bookmarklet token is in {}",
            h.join("clip.token").display()
        );
        return server::serve(&addr, home.map(String::from), url.clone());
    }
    let mut lib = Library::open(home, c.ocr)?;
    match c.cmd {
        Cmd::Bookmarklet { port } => {
            let tok = server::token(&lib.home);
            println!(
                "javascript:(async()=>{{try{{const s=String(getSelection());const r=await fetch('http://127.0.0.1:{port}/carta/clip?token={tok}',{{method:'POST',headers:{{'Content-Type':'text/plain'}},body:JSON.stringify({{url:location.href,title:document.title,text:s||document.body.innerText}})}});const j=await r.json();alert(r.ok?'Saved to CartaColibri ('+j.chars+' characters)':'CartaColibri: '+j.error)}}catch(e){{alert('CartaColibri is not running on 127.0.0.1:{port}: '+e)}}}})()"
            );
        }
        Cmd::Add { folder } => {
            println!("registered {}", lib.add_root(&folder)?);
            println!(
                "{:?}",
                lib.update(Some(&[folder]), &mut |m| eprintln!("{m}"))?
            );
        }
        Cmd::Remove { folder } => println!(
            "forgot {folder} ({} documents dropped from the index; files untouched)",
            lib.remove_root(&folder)?
        ),
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
        Cmd::List { group } => {
            for d in lib.documents(group.as_deref())? {
                println!("{:>7} KB  {:<6} {}", d.size / 1024, d.ext, d.path)
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
        } => {
            let f = Filter {
                ext: csv(&ext),
                min_size: min_kb.map(|k| (k * 1024.0) as i64),
                max_size: max_kb.map(|k| (k * 1024.0) as i64),
                path_like: path,
                group,
                limit: Some(limit),
            };
            let hits = lib.search(&query, &f)?;
            for h in &hits {
                println!(
                    "{:>7}  {} {}\n         {}",
                    h.score, h.path, h.loc, h.snippet
                );
            }
            if hits.is_empty() {
                println!("no matches");
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
    println!("{text}");
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
