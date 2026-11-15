# Jera

Eigen Mac-app waarin ik mijn Claude Code-sessies draai en overzie: sessies met status, bestanden, poorten en git in één venster. Geen browser, geen viewers, geen IDE.

## Status

Bouwfase, eerste versie. Wat besloten is, staat in `SPEC.md`. De volgorde van bouwen: geraamte, sessies met status, git-paneel, Bestanden, Render-regel, herstart.

## Stack

Besloten op 2026-11-14, zie `SPEC.md`. De feiten staan in `research/native.md`.

- GPUI met GPUI Kit (Rust).
- Terminal: `alacritty_terminal` als kern, met een zelfgebouwd tekenonderdeel in GPUI.

## Bestanden

- `SPEC.md`: het doel, de gemeten werkwijze en wat besloten is.
- `app/`: de app. Starten: `~/.cargo/bin/cargo run --manifest-path app/Cargo.toml -- [map ...]`; elke map krijgt een Claude-sessie. Als app: `app/bundel.sh` bouwt `/Applications/Jera.app` met het icoon. Kleuren en maten staan in `src/thema.rs`, overgenomen uit `stijl/design-system/tokens.json`. Het startcommando staat in `src/venster.rs`. Met `--features schermafdruk` bewaart het venster zichzelf als afbeelding (`JERA_SHOT`, `JERA_THEMA=donker|licht`, `JERA_TYPE`), zie `src/main.rs`. JetBrains Mono staat in `fonts/` (OFL). Het Dock-icoon (`iconen/dock.png` en `dock-licht.png`, uit de svg ernaast) zet `main.rs`; het teken rechtsboven is `iconen/jera.svg`. De iconen staan in `iconen/`: de eigen iconen, die van GitLab (bestanden) en Catppuccin (mappen) met hun licentie, en die van Zed (licentie niet per icoon vermeld); welk bestand welk icoon krijgt staat in `src/bestandsicoon.rs`. Wat het menu van een twee-vingerklik in Bestanden met bestanden doet (kopiëren, plakken, prullenmand, hernoemen, nieuw) staat in `src/bewerk.rs`. Het git-paneel leest de branches met git in `src/git.rs` en tekent ze in `src/gitpaneel.rs`. De sessiestatus leest `src/status.rs`, de oude sessies `src/oud.rs`, volgens `research/sessies.md`. Het scherm Openen (open, clone repository via `gh`, de mappen die ik het laatst in Jera opende, en nieuw project met ⌘N) staat in `src/openen.rs`; de recente mappen staan in `~/Library/Application Support/Jera/recent.json`. Zonder map bij de start staat Openen open. De agents van een sessie (subagents en teamleden) leest `src/agents.rs`, volgens `research/agents.md`. De instellingen staan in de menubalk (`src/instellingen.rs`) en in `~/Library/Application Support/Jera/instellingen.json`.
- `indeling/editor.html`: de editor waarin ik de indeling teken (https://claude.ai/artifact/8eNKyZX3XSg83T7d9ejG8J). De varianten staan in de opslag van dat artifact, collectie `varianten`; lees ze met ArtifactData.
- `stijl/design-system/`: het design system (https://claude.ai/artifact/QKTtDToW7LYRivNM1vYG88). De kleuren en maten staan in `tokens.json`, de onderdelen in `components/`, met een voorbeeld per onderdeel. Lees `README.md` daarvan voordat je een scherm of onderdeel ontwerpt of bouwt. Dit is de bron: verander het hier en publiceer het artifact daarna opnieuw.
- `stijl/icoon/`: het teken ᛃ, donker en licht.
- `research/` bevat het onderzoek met bronnen:
  - `native.md` (2026-11-14): GPUI met GPUI Kit, Swift, Rune en andere native app-bases, en de terminalpakketten voor GPUI.
  - `agents.md` (2026-11-14): subagents en teamleden van een sessie, waar ze staan en hoe hun stand af te lezen is. Gemeten.
  - `sessies.md` (2026-11-14): hoe Jera van buiten ziet wat een Claude Code-sessie doet: het sessiebestand, hooks, oude sessies. Gemeten.
