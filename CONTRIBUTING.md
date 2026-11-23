# Contributing

CartaColibri is a **clean-room** Rust project under the MIT licence. Keep it that way:

- Do not copy code from Colibrì (Apache-2.0), DocFetcher (EPL), Falkon (GPL-3.0) or any other project.
  Work from public specifications and documented behaviour, and say in the PR what you based a feature on.
- By sending a contribution you agree to license it under MIT.

## Workflow

```bash
cargo fmt
cargo clippy --all-targets
cargo test                                  # needs no model and no network
python3 scripts/gen_carta_docs.py           # after changing README texts (one table drives all 8 languages)
node scripts/check_carta_ui.js              # UI syntax + 8-language completeness (Node 24+)
```

- Languages: EN RO FR IT ES PT DE HU, informal address ("tu") everywhere. Add a language in
  `scripts/gen_carta_docs.py`, `scripts/gen_carta_tutorial.py` and the `D` table of `assets/carta.html`.
- Commits: `alexio@trom.tf`-style identities are fine; keep messages short and in English.
- The UI talks only to its own server; never add a call to a third-party host.
