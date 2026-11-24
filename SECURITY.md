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

## Hardening history and what remains

Earlier versions of this file listed five known gaps. All five now have a fix and a regression test in
`tests/security.rs` (no `#[ignore]`d test is left):

| Former gap | Now |
|---|---|
| Web-grab could be pointed at loopback / private / link-local hosts (SSRF), also through redirects | Refused, including tricky address forms and every redirect hop; `CARTACOLIBRI_GRAB_LOCAL=1` opts in |
| Request bodies silently truncated at the cap | Rejected with `413` |
| No `Content-Security-Policy` / `frame-ancestors` | CSP, `X-Frame-Options: DENY`, `nosniff`, `no-store`, `Cross-Origin-Resource-Policy: same-origin` on every response |
| `clip.token` created with the default umask; weak seed on Windows | `0600` on Unix, created from the OS random source |
| Browser overlay nonce discoverable by a hostile page | Per-run secrets are not exposed as page globals; `--selftest-forge` (run in CI) proves forged IPC is ignored |

Still accepted, by design or for lack of a better answer (see `docs/threat-model.md`):

1. On Windows `clip.token` relies on the ACL of your user profile; no extra permission step is taken.
2. Any local process running as you can call the API (`X-Colibri-UI` is a guard against web pages, not a secret).
3. Indexing follows symlinks, including ones that point outside the chosen folder.
4. One OS thread per connection: slow clients can exhaust threads on a shared machine.

## Dependencies

`cargo audit` and `cargo deny` run on every push and weekly (`.github/workflows/audit.yml`).
