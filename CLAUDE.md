# Cockpit

Eigen Mac-app voor mijn ontwikkelwerk: Claude-terminals, servers, browser, documenten, git en deploys in één venster. Geen IDE.

## Status

Spec-fase. De enige code is de proef in `proef/terminal`. Wat besloten is, staat in `SPEC.md`.

## Stack

Besloten op 2026-11-14, zie `SPEC.md`. De feiten staan in `research/native.md`.

- GPUI met GPUI Kit (Rust).
- Terminal: `alacritty_terminal` als kern, met een zelfgebouwd tekenonderdeel in GPUI.

## Bestanden

- `SPEC.md`: het doel, de gemeten werkwijze en wat besloten is.
- `indeling/editor.html`: de editor waarin ik de indeling teken (https://claude.ai/artifact/8eNKyZX3XSg83T7d9ejG8J). De varianten staan in de opslag van dat artifact, collectie `varianten`; lees ze met ArtifactData.
- `git-graaf/`: ontwerp van het git-paneel als werkend HTML-voorbeeld (https://claude.ai/artifact/BT7Ues1JGeMQKJj5Be5AWQ). `paneel.html` is het paneel zelf. `python3 git-graaf/lees_repo.py <pad naar repo>` leest de branches van een repo naar `data/`; `python3 git-graaf/bouw.py` maakt daarvan `graaf.html` (voor het artifact) en `test-graaf.html` (lokaal te openen).
- `proef/terminal`: proef die Claude Code draait in een zelfgetekende terminal (GPUI Kit en `alacritty_terminal`). Starten: `~/.cargo/bin/cargo run --manifest-path proef/terminal/Cargo.toml`. De uitkomst staat in `research/native.md`.
- `research/` bevat het onderzoek met bronnen:
  - `native.md` (2026-11-14): GPUI met GPUI Kit, Swift, Rune en andere native app-bases, en de terminalpakketten voor GPUI.
  - `werkwijze.md` (2026-11-04): hoe ik nu werk, gemeten.
  - `bestaande-tools.md` (2026-11-04): wat er al bestaat.
  - `zelf-bouwen.md` (2026-11-04): app-bases, onderdelen, risico's. De techniek per onderdeel gaat uit van Electron.
  - `viewers.md` (2026-11-04): npm-pakketten voor md, pdf, csv, xlsx, afbeeldingen, code en diff. Werken niet in GPUI.
  - `ui-basis.md` (2026-11-04): npm-pakketten voor bestandsboom, panelen, UI-primitieven, palette, iconen, git en lettertypen. Werken niet in GPUI.
