//! More commands: new RAG-CARTA tasks, chat, tags/stars/notes, saved searches, export, backup,
//! stats, imports and feeds. Wired into `main.rs` with `#[command(flatten)]`.

use cartacolibri::carta::{self, Backend, ChatSession, Options, RunExtra};
use cartacolibri::library::{Filter, Library};
use cartacolibri::{feeds, import, organize};
use clap::{Args, Subcommand};
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

type Res<T> = Result<T, String>;

#[derive(Args, Clone)]
pub struct Src {
    /// documents of this group
    #[arg(long)]
    pub group: Option<String>,
    /// documents matching this search
    #[arg(long)]
    pub query: Option<String>,
    /// documents carrying this tag
    #[arg(long)]
    pub tag: Option<String>,
}

#[derive(Args, Clone)]
pub struct Model {
    #[arg(long, env = "COLI_URL", hide_env_values = true, default_value = carta::DEFAULT_URL)]
    pub url: String,
    #[arg(long)]
    pub model: Option<String>,
    /// offline extractive fallback (no model)
    #[arg(long)]
    pub mock: bool,
    /// if the model is unreachable, fall back to the offline mode and say so
    #[arg(long)]
    pub fallback: bool,
}

impl Model {
    fn backend(&self) -> Backend {
        let be = if self.mock {
            Backend::mock()
        } else {
            Backend::gateway(&self.url, self.model.clone())
        };
        be.with_fallback(self.fallback)
    }
}

#[derive(Subcommand, Clone)]
pub enum More {
    /// run any RAG-CARTA task: compare, timeline, glossary, outline, actions, faq, flashcards, quiz, translate, explain, contradictions (or summarize/extract/synthesize)
    Task {
        /// task name
        name: String,
        /// what you want (for translate: the target language)
        request: Option<String>,
        #[command(flatten)]
        src: Src,
        #[command(flatten)]
        model: Model,
        /// language of the answer (default: the language of your request)
        #[arg(long)]
        lang: Option<String>,
        /// mark sentences whose [S#] citation is not backed by the text
        #[arg(long)]
        verify: bool,
        /// save the result: md, html or txt
        #[arg(long)]
        save: Option<String>,
    },
    /// talk to your library: every answer is retrieved from your documents and cited
    Chat {
        #[arg(long)]
        group: Option<String>,
        #[command(flatten)]
        model: Model,
        #[arg(long)]
        lang: Option<String>,
    },
    /// tags on documents
    #[command(subcommand)]
    Tag(TagCmd),
    /// star or unstar a document
    Star {
        path: String,
        #[arg(long)]
        off: bool,
    },
    /// show or set the note of a document
    Note { path: String, text: Option<String> },
    /// saved searches (smart collections)
    #[command(subcommand)]
    Saved(SavedCmd),
    /// documents you opened lately / most often
    Recent {
        #[arg(long, default_value_t = 10)]
        n: usize,
    },
    /// documents related to this one
    Related {
        path: String,
        #[arg(long, default_value_t = 8)]
        k: usize,
    },
    /// export a group, a saved search or a query: md, csv, json or zip
    Export {
        #[command(flatten)]
        src: Src,
        #[arg(long)]
        saved: Option<String>,
        #[arg(long, default_value = "md", value_parser = ["md", "csv", "json", "zip"])]
        format: String,
        /// file to write (stdout for md/csv/json when omitted; required for zip)
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// merge tags/stars/notes from a JSON export back in
    ImportJson { file: PathBuf },
    /// back up your library state to one zip
    Backup {
        out: PathBuf,
        /// include the search index (bigger, restores instantly)
        #[arg(long)]
        index: bool,
    },
    /// check a backup zip without restoring it
    BackupCheck { zip: PathBuf },
    /// restore a backup zip into the library home (the app must not be running)
    Restore { zip: PathBuf },
    /// check the index and optionally rebuild it
    Verify {
        #[arg(long)]
        repair: bool,
    },
    /// numbers about your library
    Stats,
    /// documents by month
    Timeline,
    /// bring things in: bookmarks .html, Obsidian folder, .bib, Pocket/Raindrop .csv, Evernote .enex
    Import { src: PathBuf },
    /// RSS/Atom subscriptions saved as notes
    #[command(subcommand)]
    Feeds(FeedCmd),
}

#[derive(Subcommand, Clone)]
pub enum TagCmd {
    Add {
        path: String,
        tags: Vec<String>,
    },
    Rm {
        path: String,
        tags: Vec<String>,
    },
    List,
    /// documents with this tag
    Show {
        tag: String,
    },
}

#[derive(Subcommand, Clone)]
pub enum SavedCmd {
    Add {
        name: String,
        query: String,
        #[arg(long)]
        tag: Option<String>,
        #[arg(long)]
        group: Option<String>,
        #[arg(long)]
        starred: bool,
    },
    List,
    Run {
        name: String,
    },
    Rm {
        name: String,
    },
}

#[derive(Subcommand, Clone)]
pub enum FeedCmd {
    Add {
        url: String,
    },
    List,
    Rm {
        url: String,
    },
    /// fetch every feed once and index the new items
    Poll,
    /// import subscriptions from an OPML file
    OpmlImport {
        file: PathBuf,
    },
    OpmlExport {
        out: Option<PathBuf>,
    },
}

fn feeds_file(lib: &Library) -> PathBuf {
    lib.home.join("feed-urls.json")
}
fn feed_urls(lib: &Library) -> Vec<String> {
    std::fs::read_to_string(feeds_file(lib))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}
fn set_feed_urls(lib: &Library, v: &[String]) -> Res<()> {
    // write-then-rename so an interrupted write never leaves a half-written list
    let p = feeds_file(lib);
    let tmp = p.with_extension(format!("json.{}.tmp", std::process::id()));
    std::fs::write(&tmp, serde_json::to_string_pretty(v).unwrap_or_default())
        .and_then(|_| std::fs::rename(&tmp, &p))
        .map_err(|e| {
            let _ = std::fs::remove_file(&tmp);
            e.to_string()
        })
}

/// A document the user names on the command line must be in the library (honest error otherwise).
fn known_doc(lib: &Library, path: &str) -> Res<()> {
    if lib.documents(None)?.iter().any(|d| d.path == path) {
        Ok(())
    } else {
        Err(format!(
            "`{path}` is not in the library (use the exact path shown by `cartacolibri list`)"
        ))
    }
}

fn docs_of(lib: &Library, s: &Src) -> Res<Vec<cartacolibri::library::Doc>> {
    if let Some(g) = &s.group {
        return lib.resolve_group(g);
    }
    if let Some(t) = &s.tag {
        let ids: Vec<i64> = organize::search_tagged(lib, "", &Filter::default(), Some(t), false)?
            .iter()
            .map(|h| h.id)
            .collect();
        return Ok(lib.docs_by_id(&ids));
    }
    if let Some(q) = &s.query {
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
        return Ok(lib.docs_by_id(&ids));
    }
    Err("give --group NAME, --tag TAG or --query TEXT".into())
}

fn items_of(lib: &Library, s: &Src, saved: &Option<String>) -> Res<Vec<organize::ExportItem>> {
    if let Some(n) = saved {
        return organize::items_from_saved(lib, n);
    }
    if let Some(g) = &s.group {
        return organize::items_from_group(lib, g);
    }
    Ok(organize::items_from_docs(lib, &docs_of(lib, s)?))
}

fn pretty<T: serde::Serialize>(v: &T) {
    println!("{}", serde_json::to_string_pretty(v).unwrap_or_default());
}

/// `restore` must run before a library is opened on the same home.
pub fn restore(home: Option<&str>, zip: &Path) -> Res<()> {
    let h = cartacolibri::library::state_dir(home)?;
    let r = organize::restore(zip, &h)?;
    println!(
        "restored {} file(s){}",
        r.files,
        if r.needs_update {
            "; run `cartacolibri index` to rebuild the index"
        } else {
            ""
        }
    );
    Ok(())
}

/// `verify` works from the home folder, so `--repair` can also fix an index that cannot be opened.
pub fn verify_cmd(home: Option<&str>, repair: bool) -> Res<()> {
    let h = cartacolibri::library::state_dir(home)?;
    let (_lib, r) = organize::verify_home(&h.to_string_lossy(), repair)?;
    println!(
        "{}",
        if r.ok {
            "index OK"
        } else {
            "index has problems"
        }
    );
    r.problems.iter().for_each(|p| println!("  - {p}"));
    if r.rebuilt {
        println!("index rebuilt: {} documents", r.docs);
    }
    if !r.ok {
        return Err("the index is damaged: run `cartacolibri verify --repair`".into());
    }
    Ok(())
}

pub fn run(lib: &mut Library, cmd: More) -> Res<()> {
    match cmd {
        More::Task {
            name,
            request,
            src,
            model,
            lang,
            verify,
            save,
        } => {
            let docs = docs_of(lib, &src)?;
            let req = request.unwrap_or_default();
            let o = Options {
                request: &req,
                ..Default::default()
            };
            let x = RunExtra {
                answer_lang: lang,
                target_lang: if name == "translate" {
                    Some(req.clone()).filter(|r| !r.is_empty())
                } else {
                    None
                },
                verify,
                ..Default::default()
            };
            let text = carta::run_ex(lib, &name, &docs, &model.backend(), &o, &x, &mut |m| {
                eprintln!("· {m}")
            })?;
            println!("{}", cartacolibri::tour::plain(&text));
            if let Some(fmt) = save {
                let g = src
                    .group
                    .clone()
                    .or(src.tag.clone())
                    .unwrap_or_else(|| "selection".into());
                println!(
                    "saved: {}",
                    carta::save_as(&lib.home, &text, &name, &g, &req, &fmt)?
                );
            }
        }
        More::Chat { group, model, lang } => {
            let mut s = ChatSession::new(model.backend());
            s.group = group;
            s.answer_lang = lang;
            eprintln!(
                "Ask about your documents. Empty line or Ctrl+D to leave, /new to forget the conversation."
            );
            let stdin = std::io::stdin();
            loop {
                eprint!("› ");
                let _ = std::io::stderr().flush();
                // bytes, then lossy: an invalid UTF-8 byte must not end the conversation
                let mut raw = Vec::new();
                if stdin
                    .lock()
                    .read_until(b'\n', &mut raw)
                    .map_err(|e| e.to_string())?
                    == 0
                {
                    break;
                }
                let l = String::from_utf8_lossy(&raw).into_owned();
                let q = l.trim();
                if q.is_empty() {
                    break;
                }
                if q == "/new" {
                    s.reset();
                    continue;
                }
                match s.ask(lib, q) {
                    Ok(r) => println!("{}\n", cartacolibri::tour::plain(&r.render())),
                    Err(e) => eprintln!("{e}"),
                }
            }
        }
        More::Tag(t) => match t {
            TagCmd::Add { path, tags } => {
                known_doc(lib, &path)?;
                println!("{}", organize::tag_add(lib, &path, &tags)?.join(", "))
            }
            TagCmd::Rm { path, tags } => {
                println!("{}", organize::tag_remove(lib, &path, &tags)?.join(", "))
            }
            TagCmd::List => {
                for (t, n) in organize::tag_list(lib) {
                    println!("{n:>5}  {t}");
                }
            }
            TagCmd::Show { tag } => organize::paths_with_tag(lib, &tag)
                .iter()
                .for_each(|p| println!("{p}")),
        },
        More::Star { path, off } => {
            if !off {
                known_doc(lib, &path)?;
            }
            organize::set_star(lib, &path, !off)?;
            println!("{} {path}", if off { "unstarred" } else { "★" });
        }
        More::Note { path, text } => match text {
            Some(t) => {
                known_doc(lib, &path)?;
                organize::set_note(lib, &path, &t)?;
                println!("note saved");
            }
            None => println!("{}", organize::note_of(lib, &path)),
        },
        More::Saved(s) => match s {
            SavedCmd::Add {
                name,
                query,
                tag,
                group,
                starred,
            } => {
                organize::saved_set(
                    lib,
                    organize::SavedSearch {
                        name: name.clone(),
                        query,
                        tag,
                        group,
                        starred_only: starred,
                        ..Default::default()
                    },
                )?;
                println!("saved search “{name}”");
            }
            SavedCmd::List => {
                for s in organize::saved_list(lib) {
                    println!("{}  —  {}", s.name, s.query);
                }
            }
            SavedCmd::Run { name } => {
                for h in organize::saved_run(lib, &name)? {
                    println!("{:>3}%  {}\n      {}", h.score, h.path, h.snippet);
                }
            }
            SavedCmd::Rm { name } => println!(
                "{}",
                if organize::saved_remove(lib, &name)? {
                    "removed"
                } else {
                    "no such saved search"
                }
            ),
        },
        More::Recent { n } => {
            for p in organize::recent(lib, n) {
                println!("{p}");
            }
            for (p, c) in organize::most_opened(lib, n) {
                println!("{c:>4}×  {p}");
            }
        }
        More::Related { path, k } => {
            let id = lib
                .documents(None)?
                .into_iter()
                .find(|d| d.path == path)
                .map(|d| d.id)
                .ok_or("that document is not in the library")?;
            for h in organize::related(lib, id, k)? {
                println!("{:>3}%  {}", h.score, h.path);
            }
        }
        More::Export {
            src,
            saved,
            format,
            out,
        } => {
            let items = items_of(lib, &src, &saved)?;
            if format == "zip" {
                let o = out.ok_or("--out FILE.zip is required for zip")?;
                println!(
                    "{} source file(s) bundled into {}",
                    organize::export_zip(&items, &o)?,
                    o.display()
                );
            } else {
                let t = match format.as_str() {
                    "csv" => organize::to_csv(&items),
                    "json" => organize::to_json(&items),
                    _ => organize::to_markdown(&items),
                };
                match out {
                    Some(o) => std::fs::write(&o, t).map_err(|e| e.to_string())?,
                    None => println!("{t}"),
                }
            }
        }
        More::ImportJson { file } => {
            let t =
                std::fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
            pretty(&organize::import_json(lib, &t)?);
        }
        More::Backup { out, index } => {
            let m = organize::backup(lib, &out, index)?;
            println!(
                "backup written: {} ({} files)",
                out.display(),
                m.files.len()
            );
        }
        More::BackupCheck { zip } => {
            let m = organize::backup_verify(&zip)?;
            println!(
                "OK — {} files, made by version {}",
                m.files.len(),
                m.app_version
            );
        }
        More::Restore { .. } => unreachable!(),
        More::Verify { repair } => {
            let r = organize::verify(lib, repair)?;
            println!(
                "{}",
                if r.ok {
                    "index OK"
                } else {
                    "index has problems"
                }
            );
            r.problems.iter().for_each(|p| println!("  - {p}"));
            if r.rebuilt {
                println!("index rebuilt: {} documents", r.docs);
            }
        }
        More::Stats => pretty(&organize::stats(lib)?),
        More::Timeline => {
            for m in organize::timeline(lib)? {
                println!("{}  {:>4}", m.month, m.count);
            }
        }
        More::Import { src } => {
            let dest = lib.home.join("library").join("uploads").join("imported");
            let r = import::import_path(&src, &dest)?;
            println!("{} note(s) written to {}", r.notes, r.dest);
            lib.add_root(&dest.to_string_lossy())?;
            let st = lib.update(Some(&[dest.to_string_lossy().into_owned()]), &mut |_| {})?;
            println!("indexed: {} new/changed", st.added + st.updated);
        }
        More::Feeds(f) => match f {
            FeedCmd::Add { url } => {
                let mut v = feed_urls(lib);
                if !v.contains(&url) {
                    v.push(url.clone());
                }
                set_feed_urls(lib, &v)?;
                println!("subscribed to {url}");
            }
            FeedCmd::List => feed_urls(lib).iter().for_each(|u| println!("{u}")),
            FeedCmd::Rm { url } => {
                let v: Vec<String> = feed_urls(lib).into_iter().filter(|u| *u != url).collect();
                set_feed_urls(lib, &v)?;
                println!("unsubscribed from {url}");
            }
            FeedCmd::Poll => {
                let urls = feed_urls(lib);
                let home = lib.home.clone();
                pretty(&feeds::poll(&home, &urls, lib));
            }
            FeedCmd::OpmlImport { file } => {
                let t = std::fs::read_to_string(&file).map_err(|e| e.to_string())?;
                let mut v = feed_urls(lib);
                for s in feeds::opml_parse(&t) {
                    if !v.contains(&s.url) {
                        v.push(s.url);
                    }
                }
                set_feed_urls(lib, &v)?;
                println!("{} subscription(s)", v.len());
            }
            FeedCmd::OpmlExport { out } => {
                let subs: Vec<feeds::Subscription> = feed_urls(lib)
                    .into_iter()
                    .map(|u| feeds::Subscription {
                        title: u.clone(),
                        url: u,
                    })
                    .collect();
                let x = feeds::opml_export(&subs);
                match out {
                    Some(o) => std::fs::write(o, x).map_err(|e| e.to_string())?,
                    None => println!("{x}"),
                }
            }
        },
    }
    Ok(())
}
