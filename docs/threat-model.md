# CartaColibri threat model

## Assets

* The user's documents and the extracted text in the SQLite index (`~/.cartacolibri`).
* Saved web pages, highlights, history and watch lists in the same folder.
* The clip token (`clip.token`) and the browser's per-run IPC nonce.
* The user's other local services (reachable from the machine, not from the internet).
* Availability of the user's machine (disk, memory, CPU).

## Trust boundaries

1. Web page in the user's browser (or in the CartaColibri browser) -> local UI server on 127.0.0.1.
2. Local UI server -> file system (user folders, upload folder, library home).
3. Local UI server / web-grab -> the network (page fetches) and the local model gateway (Colibri).
4. Visited web page -> browser overlay (injected script) -> Rust IPC handler.
5. Hostile file (zip, xml, pdf, html) -> extractors.

## Attackers

| Attacker | Capability |
|---|---|
| Hostile website | Makes the user's browser send requests to `127.0.0.1:<port>`; can rebind DNS. |
| Hostile document | Delivered by email or download; opened by indexing a folder. |
| Hostile page inside the CartaColibri browser | Runs script next to the injected overlay. |
| Other local user | Reads files with default permissions on a shared machine. |
| Network attacker | Cannot reach the server (loopback only); can tamper with plain-http grabs. |

Out of scope: malware already running as the user.

## Mitigations

| Threat | Mitigation | Test |
|---|---|---|
| DNS rebinding | `Host` must be `127.0.0.1`, `localhost` or `[::1]` on every route | `bad_host_is_rejected` |
| CSRF / cross-site calls | `Origin` must equal the page origin; `X-Colibri-UI` required on `/carta/*`; no CORS headers there (preflight fails) | `bad_origin_is_rejected`, `carta_routes_need_the_ui_header` |
| Bookmarklet abuse | `/carta/clip` needs a 160-bit token, POST only; Host still checked | `clip_route_requires_the_token` |
| Path traversal | Static routes are a fixed allow-list; `doc` takes a numeric id; upload names are reduced to a base name with a safe character set | `path_traversal_attempts_are_rejected`, `upload_filenames_cannot_escape_the_uploads_folder` |
| Scheme abuse in grab | Only `http://` / `https://` accepted | `grab_refuses_non_http_schemes` |
| SSRF through grab / clip | Loopback, private, link-local and tunnelled-IPv4 targets are refused, redirects are re-checked on every hop; `CARTACOLIBRI_GRAB_LOCAL=1` opts in | `grab_refuses_loopback_and_private_targets`, `ssrf_address_classifier_covers_tricky_forms`, `redirect_targets_resolve_like_a_browser` |
| Oversized bodies | Refused with `413`, never truncated | `oversized_body_is_rejected` |
| Framing / injected content | CSP (`default-src 'none'`, same-origin `connect-src`), `X-Frame-Options: DENY`, `Cross-Origin-Resource-Policy` | `ui_page_forbids_framing`, `ui_page_has_a_locked_down_csp` |
| Token disclosure | `clip.token` is mode 0600 and excluded from backups | `token_file_is_private`, `backup_does_not_contain_the_clip_token` |
| Zip bombs | Each zip part capped at 100 MiB (`MAX_ZIP_PART`), read with `take` | unit tests in extract |
| Parser crashes | Extraction runs under `catch_unwind` | - |
| Hostile JSON | `serde_json` recursion limit (128) turns deep nesting into a 400 | `malformed_and_hostile_json_does_not_crash_the_server` |
| XSS | The UI escapes `& < > " '` before inserting any file name, title, snippet or model output; attributes are double-quoted; overlay UI is in a closed shadow root and uses `textContent` | review |
| Browser IPC | Messages without the per-run nonce are dropped; `newtab` accepts http(s) only | review |
| Content sniffing / caching | `nosniff`, `no-store` on every response | `responses_are_not_cacheable_or_sniffable` |

## Residual risks

See `SECURITY.md` for the table of former gaps (all fixed) and the accepted items. Further accepted risks:

* Any local process can call the API (`X-Colibri-UI` is not a secret); local-only is the design.
* Indexing follows symlinks, including files that point outside the chosen folder.
* Zip archives are capped per part and per entry count and by what is really written, not by a global memory budget.
* On Windows `clip.token` is protected by the profile's ACL only.
* One OS thread per connection; slow clients can exhaust threads on a shared machine.
