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
- `app/`: de app. Starten: `~/.cargo/bin/cargo run --manifest-path app/Cargo.toml -- [map ...]`; elke map krijgt een Claude-sessie. Als app: `app/bundel.sh` bouwt `/Applications/Jera.app` met het icoon. Ontwikkelen terwijl Jera open is: `app/test.sh [map ...]` start een test-Jera naast de gewone en start hem opnieuw zodra de code verandert (`watchexec`). Een debug-build is de test-Jera: `test` op het Dock-icoon en de gegevens in `Application Support/Jera-test` (`gegevens()` in `src/main.rs`). Kleuren en maten staan in `src/thema.rs`, overgenomen uit `stijl/design-system/tokens.json`. Het startcommando staat in `src/venster.rs`. Met `--features schermafdruk` bewaart het venster zichzelf als afbeelding (`JERA_SHOT`, `JERA_THEMA=donker|licht`, `JERA_TYPE`), zie `src/main.rs`. JetBrains Mono staat in `fonts/` (OFL). Het Dock-icoon (`iconen/dock.png` en `dock-licht.png`, uit de svg ernaast) zet `main.rs`; het teken rechtsboven is `iconen/jera.svg`. De iconen staan in `iconen/`: de eigen iconen, die van GitLab (bestanden) en Catppuccin (mappen) met hun licentie, en die van Zed (met de licentie die Zed erbij geeft, Lucide); welk bestand welk icoon krijgt staat in `src/bestandsicoon.rs`. Wat het menu van een twee-vingerklik in Bestanden met bestanden doet (kopiëren, pad kopiëren, plakken, prullenmand, hernoemen, nieuw) en wat slepen doet staat in `src/bewerk.rs`: slepen in Bestanden verplaatst, een bestand uit Finder in Bestanden of in de sessielijst wordt gekopieerd (boven de sessielijst staat dan het vlak `toevoegen_vlak` in `src/venster.rs`), daarna staat Bestanden open met het nieuwe bestand gekozen en het pad vanaf de projectmap op het klembord. Het git-paneel leest de branches met git in `src/git.rs` en tekent ze in `src/gitpaneel.rs`. De sessiestatus leest `src/status.rs`, de oude sessies `src/oud.rs`, volgens `research/sessies.md`; waar Claude Code zijn bestanden bewaart staat in `src/claude.rs`. Het venster is opgedeeld in `src/venster.rs` en `src/venster/` (sessielijst, bestandsboom, naamveld en de gedeelde onderdelen). Het scherm Openen (open, clone repository via `gh`, de mappen die ik het laatst in Jera opende, en nieuw project met ⌘N) staat in `src/openen.rs`; de recente mappen staan in `~/Library/Application Support/Jera/recent.json`. Zonder map bij de start staat Openen open. De agents van een sessie (subagents en teamleden) leest `src/agents.rs`, volgens `research/agents.md`. De poorten en agent-browsers met de sessie die ze startte leest `src/poorten.rs`, volgens `research/poorten.md`; het paneel ervan staat in `src/venster/poortenpaneel.rs`. Daarin staan onder elke sessie ook de artifacts die ze publiceerde; die leest `src/artifacts.rs` uit haar gesprek (`toolUseResult`), de keuze staat in `stijl/artifacts/`. Het venster Documenten (markdown, csv en xlsx, json, afbeeldingen en pdf, één tab per bestand, opnieuw gelezen als het bestand verandert) staat in `src/documenten.rs` en `src/documenten/`; een klik in Bestanden en een ⌘-klik op een pad in de terminal gaan erheen via `documenten::open`, wat het niet toont gaat naar de standaard-app. Met `--features schermafdruk` opent `JERA_DOC=pad|pad` bestanden bij de start en bewaart `JERA_DOC_SHOT` dat venster als afbeelding. De instellingen staan in de menubalk (`src/instellingen.rs`) en in `~/Library/Application Support/Jera/instellingen.json`. De taal (Nederlands of Engels) staat in `src/taal.rs`: elke tekst in beeld gaat door `taal::t("nl", "en")`.
- `indeling/editor.html`: de editor waarin ik de indeling teken (https://claude.ai/artifact/8eNKyZX3XSg83T7d9ejG8J). De varianten staan in de opslag van dat artifact, collectie `varianten`; lees ze met ArtifactData.
- `stijl/design-system/`: het design system (https://claude.ai/artifact/QKTtDToW7LYRivNM1vYG88). De kleuren en maten staan in `tokens.json`, de onderdelen in `components/`, met een voorbeeld per onderdeel. Lees `README.md` daarvan voordat je een scherm of onderdeel ontwerpt of bouwt. Dit is de bron: verander het hier en publiceer het artifact daarna opnieuw.
- `stijl/icoon/`: het teken ᛃ, donker en licht.
- `research/` bevat het onderzoek met bronnen:
  - `native.md` (2026-11-14): GPUI met GPUI Kit, Swift, Rune en andere native app-bases, en de terminalpakketten voor GPUI.
  - `agents.md` (2026-11-14): subagents en teamleden van een sessie, waar ze staan en hoe hun stand af te lezen is. Gemeten.
  - `sessies.md` (2026-11-14): hoe Jera van buiten ziet wat een Claude Code-sessie doet: het sessiebestand, hooks, oude sessies. Gemeten.
  - `poorten.md` (2026-11-15): welke sessie een poort of agent-browser startte (via het sessie-id in de omgeving), en de bestanden van agent-browser. Gemeten.
