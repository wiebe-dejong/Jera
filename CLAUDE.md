# Cockpit

Eigen Mac-app waarin ik mijn Claude Code-sessies draai en overzie: sessies met status, bestanden, poorten en git in één venster. Geen browser, geen viewers, geen IDE.

## Status

Bouwfase, eerste versie. Wat besloten is, staat in `SPEC.md`. De volgorde van bouwen: geraamte, sessies met status, git-paneel, Bestanden, Render-regel, herstart.

## Stack

Besloten op 2026-11-14, zie `SPEC.md`. De feiten staan in `research/native.md`.

- GPUI met GPUI Kit (Rust).
- Terminal: `alacritty_terminal` als kern, met een zelfgebouwd tekenonderdeel in GPUI.

## Bestanden

- `SPEC.md`: het doel, de gemeten werkwijze en wat besloten is.
- `app/`: de app. Starten: `~/.cargo/bin/cargo run --manifest-path app/Cargo.toml -- [map ...]`; elke map krijgt een Claude-sessie. Kleuren en maten staan in `src/thema.rs`, overgenomen uit `stijl/design-system/tokens.json`. Het startcommando staat in `src/venster.rs`. Met `--features schermafdruk` bewaart het venster zichzelf als afbeelding (`COCKPIT_SHOT`, `COCKPIT_THEMA=donker|licht`, `COCKPIT_TYPE`), zie `src/main.rs`. JetBrains Mono staat in `fonts/` (OFL).
- `indeling/editor.html`: de editor waarin ik de indeling teken (https://claude.ai/artifact/8eNKyZX3XSg83T7d9ejG8J). De varianten staan in de opslag van dat artifact, collectie `varianten`; lees ze met ArtifactData.
- `git-graaf/`: ontwerp van het git-paneel als werkend HTML-voorbeeld (https://claude.ai/artifact/BT7Ues1JGeMQKJj5Be5AWQ). `paneel.html` is het paneel zelf. `python3 git-graaf/lees_repo.py <pad naar repo>` leest de branches van een repo naar `data/`; `python3 git-graaf/bouw.py` maakt daarvan `graaf.html` (voor het artifact) en `test-graaf.html` (lokaal te openen).
- `stijl/design-system/`: het design system (https://claude.ai/artifact/QKTtDToW7LYRivNM1vYG88). De kleuren en maten staan in `tokens.json`, de onderdelen in `components/`, met een voorbeeld per onderdeel. Lees `README.md` daarvan voordat je een scherm of onderdeel ontwerpt of bouwt. Dit is de bron: verander het hier en publiceer het artifact daarna opnieuw.
- `stijl/cockpit/`: het keuzebord voor het uiterlijk (`keuzebord.html`, https://claude.ai/artifact/6M7pWS1JLMd6jWtFqZkoj3) en `keuzes.md` met de stand per ronde.
- `proef/terminal`: proef die Claude Code draait in een zelfgetekende terminal (GPUI Kit en `alacritty_terminal`). Starten: `~/.cargo/bin/cargo run --manifest-path proef/terminal/Cargo.toml`. De uitkomst staat in `research/native.md`.
- `research/` bevat het onderzoek met bronnen:
  - `native.md` (2026-11-14): GPUI met GPUI Kit, Swift, Rune en andere native app-bases, en de terminalpakketten voor GPUI.
  - `werkwijze.md` (2026-11-04): hoe ik nu werk, gemeten.
  - `zelf-bouwen.md` (2026-11-04): app-bases, onderdelen, risico's. De techniek per onderdeel gaat uit van Electron.
