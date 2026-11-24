# Security policy

CartaColibri is a local-first tool: a library indexer and a small web UI that listen on
`127.0.0.1` only, plus an optional desktop browser. It is built so that nothing leaves your
machine except the page fetches you ask for.

## Reporting a vulnerability

Please report privately to **alexio@trom.tf** (subject: `CartaColibri security`). Include the
version (`cartacolibri --version`), your OS, and steps to reproduce. You can expect an
acknowledgement within 7 days and a fix or a mitigation plan within 90 days. Please do not open
a public issue until a fix is released. We credit reporters unless they prefer to stay anonymous.

## Supported versions

| Version | Supported |
|---|---|
| latest `0.2.x` pre-release / `main` | yes |
| anything older | no |

## Design in one paragraph

The UI server binds to `127.0.0.1` only. Every request must carry a loopback `Host` (defence
against DNS rebinding); an `Origin`, when present, must equal the page's own origin; every
`/carta/*` call must carry the `X-Colibri-UI` header (a cross-site form or image cannot set it,
and no CORS preflight is ever granted); the one cross-origin route, `/carta/clip`, requires a
secret token. Uploads are renamed to a safe file name inside the library folder. Zip parts are
size-capped. See `docs/threat-model.md` for the full picture.

## In scope

* The UI server (`src/server.rs`, `src/api.rs`) and its Host / Origin / header / token checks.
* Path handling in uploads, folders and the document viewer.
* Web-grab (`src/grab.rs`) and the clip bookmarklet.
* File extractors (`src/extract.rs`): crashes, hangs, memory blow-ups on hostile files.
* The browser (`browser/`): the page-to-app IPC and its nonce.
* XSS in the UI through file names, titles, snippets or model output.

## Out of scope

* An attacker who already runs code as your user, or who can read your home folder.
* Another local user on a shared machine, beyond the file-permission items listed below.
* Content of web pages you choose to grab, and the answers of the local model (Colibri).
* Denial of service by a user against their own machine.
* Vulnerabilities in third-party services you point the tool at.

## Known gaps (tracked, with `#[ignore]` tests in `tests/security.rs`)

These are documented rather than hidden. Run `cargo test --test security -- --ignored` to see
their status.

1. Web-grab does not block loopback / private / link-local targets or redirects to them (SSRF).
   A page you grab can only be fetched by you or by a holder of the clip token.
2. Request bodies are capped at 200 MiB and silently truncated rather than rejected with 413.
3. The UI page has no `Content-Security-Policy` / `frame-ancestors` (clickjacking by a framing page).
4. `clip.token` is created with the default umask (typically 0644), and on Windows its seed lacks
   a strong random source.
5. The browser overlay derives a global property name from the IPC nonce, so a hostile page can
   discover the nonce and forge toolbar commands.

## Dependencies

`cargo audit` and `cargo deny` run on every push and weekly (`.github/workflows/audit.yml`).
