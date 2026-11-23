# Contributing

CartaColibri is a **clean-room** Rust project under the MIT licence. Keep it that way:

- Do not copy code from Colibrì (Apache-2.0), DocFetcher (EPL), Falkon (GPL-3.0) or any other project.
  Work from public specifications and documented behaviour, and say in the PR what you based a feature on.
- By sending a contribution you agree to license it under MIT.

## New here?

You do not need to write Rust to help. Good first contributions:

- **Try the first 5 minutes** in the [README](README.md) on a fresh machine (especially Windows or macOS, which nobody has run by hand yet) and tell us where you got stuck.
- **Improve a translation**: the beginner texts (first steps, FAQ, troubleshooting, glossary, privacy) live in `scripts/carta_docs_beginner.py`, the rest in `scripts/gen_carta_docs.py`. Edit there, never the generated files.
- **Report a confusing error message** or a missing entry for the troubleshooting table.
- Building from source: install Rust 1.90+ and run `cargo build --release`; `cargo test` needs no model and no network.

Be kind: this is a young alpha project, and questions from beginners are welcome in issues.

## Workflow

```bash
cargo fmt
cargo clippy --all-targets
cargo test                                  # needs no model and no network
python3 scripts/gen_carta_docs.py           # after changing README texts (one table drives all 8 languages; beginner texts are in scripts/carta_docs_beginner.py)
node scripts/check_carta_ui.js              # UI syntax + 8-language completeness (Node 24+)
```

- Languages: EN RO FR IT ES PT DE HU, informal address ("tu") everywhere. Add a language in
  `scripts/gen_carta_docs.py`, `scripts/gen_carta_tutorial.py` and the `D` table of `assets/carta.html`.
- Commits: `alexio@trom.tf`-style identities are fine; keep messages short and in English.
- The UI talks only to its own server; never add a call to a third-party host.
