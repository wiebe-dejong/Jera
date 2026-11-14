# Native app-bases (2026-11-14)

Opties zonder browser in de schil: de app tekent zelf op de GPU, of gebruikt de bouwstenen van macOS.

**Legenda:**
- ✓ = zelf gecontroleerd op 2026-11-14, via de GitHub-API, crates.io of de broncode.
- (b) = claim uit de README of het plan van het project zelf, niet getest.
- Er is niets gebouwd en niets gemeten.

## Vergelijking

| Optie | Versie | Licentie | Ligt klaar | Ontbreekt |
|---|---|---|---|---|
| GPUI met GPUI Kit (Rust) | GPUI Kit 0.9.1, 9 nov 2026 ✓. Zed 1.28.1, 11 nov 2026 ✓ | Apache-2.0 ✓ | Dock met tabs en splits, bestandsboom, tabellen, code-editor, Markdown, webview (b) | Terminal-component ✓ |
| Native Swift | met macOS | Apple | Boom, tabel en splits van macOS, PDFKit, Quick Look, WKWebView | Dock met tabs, code-editor, diff |
| Rune (Rust) | geen release. Repo van 22 okt 2026 ✓ | MIT ✓ | Basiscomponenten, terminal-grid, split-diff (b) | Boom, tabel, tabs, splitters, webview ✓ |
| Slint | 1.18.1, 21 sep 2026 ✓ | eigen licenties | – | Dock, terminal, documentviewers |
| egui | 0.37.0, 15 okt 2026 ✓ | Apache-2.0 ✓ | – | idem |
| iced | 0.14.0, 7 dec 2025 ✓ | MIT ✓ | – | idem. Nog geen 1.0 |
| Xilem | 0.4.0, 29 okt 2025 ✓ | Apache-2.0 ✓ | – | Noemt zichzelf experimenteel ✓ |
| Floem | 0.2.0, 15 nov 2024 ✓ | MIT ✓ | – | Laatste push 24 sep 2026 ✓ |

## GPUI met GPUI Kit

**GPUI** is het UI-framework onder de editor Zed.
- Op crates.io staat `gpui` 0.2.2 ✓, Apache-2.0 ✓.
- GPUI Kit pint `gpui-pre` 0.3.12 (9 nov 2026) ✓. Dat is een voorversie; crates.io noemt de Zed-repo als bron ✓.

**GPUI Kit** (`longbridge/gpui-kit`, eerder gpui-component):
- 20,7k sterren, laatste push 14 nov 2026 ✓.
- Code onder Apache-2.0 (`LICENSE-APACHE`), de docs onder een eigen licentie ✓.
- Gebruikt in de commerciële app Longbridge Pro (b).
- Eén afhankelijkheid: `gpui-kit` exporteert GPUI en de componenten opnieuw (b).

Onderdelen volgens de README (b):
- ruim 75 componenten;
- dock-indeling: panelen met instelbare grootte, versleepbare tabs, geneste splits, op te slaan als data;
- tree ✓ (`crates/component/src/tree.rs`);
- tabellen met virtueel scrollen, vaste en verstelbare kolommen, sorteren, over honderdduizenden rijen;
- virtuele lijsten, ook met rijen van verschillende hoogte;
- code-editor: 200k regels, Tree-sitter, LSP;
- Markdown en HTML, syntaxkleuring, grafieken;
- toegankelijkheid via AccessKit;
- headless UI-tests: componenten renderen zonder venster, invoer afspelen, toestand en layout controleren;
- "120 FPS".

**Webview** ✓ (`crates/webview`, gebouwd op Wry):
- op macOS is het WKWebView. Dat heeft geen CDP (zie `zelf-bouwen.md`);
- de README noemt het experimenteel, met beperkte functies;
- op `gpui-pre` ligt de webview boven alle GPUI-inhoud in zijn vlak, dus ook boven popovers, dialogen, menu's en tooltips, en hij is niet te clippen;
- met de backend `gpui-fast` kunnen overlays op macOS boven de webview komen (b).

**Terminal:**
- in GPUI Kit is geen terminal-component gevonden ✓ (gezocht in de code);
- Zed gebruikt `alacritty_terminal` ✓, op crates.io 0.26.0 ✓;
- Zeds eigen terminal-crate is GPL-3.0-or-later ✓.

**Pakketten:** npm-pakketten werken niet in GPUI.

## Native Swift

- **libghostty-vt 1.0** verscheen op 6 nov 2026 met een stabiele C-API (b, uit nieuwsberichten). Dat is de emulatiekern. De volledige API om een terminal in een app in te bouwen is nog niet stabiel (b).
- **cmux** is zo gebouwd: Swift met libghostty.
- De ingebouwde browser is WKWebView, zonder CDP.

## Rune

`anomalyco/rune`, van de makers van OpenCode.
- Repo aangemaakt op 22 okt 2026, 7.085 sterren, 379 open issues ✓.
- De README zegt "Rune is pre-release" ✓. Er zijn geen releases ✓: je pint een commit van de `dev`-branch.
- Het project neemt geen pull requests aan ✓.
- **Componenten** (b): knoppen, tekstvelden, select, menu's en contextmenu's, popover, tooltip, toast, lijst, combobox met gevirtualiseerde rijen, split-diff.
- **Terminal:** het element `grid` tekent een terminalscherm, toegevoegd op 13 nov 2026 ✓ (changelog). Emulatie en PTY zitten er niet in.
- **Webview:** niet in de code ✓. Alleen in de onderzoeksnotities, als plan met WKWebView.
- **Headless** (b): elke app kan zonder venster naar PNG renderen, de elementboom tonen en invoer afspelen.
- **Mac-app:** het script `scripts/mac-app` maakt, ondertekent en installeert een `.app` ✓.
- **Doelen uit hun plan** (b), geen metingen:

  | Wat | Doel |
  |---|---|
  | Opstarten tot eerste beeld | p50 ≤ 150 ms |
  | Invoer tot nieuw beeld | p50 ≤ 8,3 ms |
  | Lijst met 1 miljoen rijen | eerste scherm ≤ 100 ms |
  | Geheugen in rust | ≤ 50 MiB |
  | Grootte van een minimale app | ≤ 5 MiB |

- **Hot reload** (b): een structurele wijziging in Rust kost ~1 s, door hen gemeten.
- **Over GPUI** (b, uit het plan van Rune): GPUI tekent bij elke wijziging het hele venster opnieuw en laat animaties via de hoofdthread lopen.

## Terminal in GPUI

Een terminal bestaat uit een kern, die bijhoudt wat er op het scherm staat, en een onderdeel dat dat scherm in GPUI tekent.

### Kern

| | `alacritty_terminal` | `libghostty-vt` |
|---|---|---|
| Herkomst | Kern van Alacritty. Zed gebruikt hem ✓ | Kern van Ghostty |
| Versie | 0.26.0, 6 apr 2026 ✓. Op crates.io sinds juli 2020, 2,7 mln downloads ✓ | 1.0.1 ✓. 1.0 op 6 nov 2026 met stabiele C-API (b) |
| Licentie | Apache-2.0 ✓ | MIT (b) |
| Taal | Rust ✓ | Zig, via een Rust-koppeling |
| Synchronized output (mode 2026) | ja ✓ (in `vte`, `ansi.rs`) | ja ✓ (`RenderHold`) |
| Kitty-toetsenbordprotocol | ja ✓ | ja ✓ (`key.rs`, `kitty.rs`) |
| OSC 8 (links) | ja ✓ | niet gecontroleerd |
| OSC 7 (werkmap) | nee ✓ | ja ✓ (`PwdChanged`) |
| OSC 9 en 777 (meldingen) | nee ✓ | ja ✓ (`DesktopNotification`) |
| OSC 9;4 (voortgang) | nee ✓ | ja ✓ (`ProgressReport`) |
| OSC 133 (prompt) | nee ✓ | scripts voor shell-integratie zitten erbij ✓ |
| Afbeeldingen | nee | Kitty graphics ✓ |

- **`alacritty_terminal`** verwerkt deze OSC-codes ✓: 0, 2, 4, 8, 10–12, 50, 52, 104, 110–112. Andere codes negeert hij; die zijn alleen uit de ruwe datastroom te lezen.
- **OSC 7501** (Claude Code) is bij geen van beide gevonden.
- **Rust-koppelingen voor `libghostty-vt`**, beide van derden:
  - `libghostty-vt` 0.2.2, 28 sep 2026, MIT. Repo `Uzaaft/libghostty-rs`: 412 sterren, 207k downloads ✓. Uitgebracht vóór 1.0.
  - `ghostty-vt` 0.9.0, 14 nov 2026, Apache-2.0. Volgt 1.0.1 ✓. Een fork van de vorige, uit een repo met 2 sterren ✓. Op macOS een vooraf gebouwde bibliotheek, elders Zig 0.17 (b).

### Tekenonderdeel

| Pakket | Versie | Licentie | Kern | GPUI-versie | Omvang |
|---|---|---|---|---|---|
| `gpui_xterm` | 0.3.0, 10 nov 2026 ✓ | MIT ✓ | `alacritty_terminal` ^0.25.1 ✓ | `gpui-kit` ^0.9.0 ✓ | 5 sterren, 170 downloads, 5 versies ✓ |
| `gpui-libghostty` | 0.4.1-rc.1, 2 nov 2026 ✓ | MIT ✓ | libghostty | via een eigen macro | 10 sterren. "Alpha" (b). Vraagt Zig 0.17 en de Xcode-tools (b) |
| `gpui-terminal` | 0.1.0, 24 dec 2025 ✓ | MIT of Apache-2.0 ✓ | `alacritty_terminal` ^0.25.1 ✓ | `gpui` ^0.2.2 ✓ | 50 sterren, 1 versie ✓ |
| `ely-terminal` | 0.1.1, 26 okt 2026 ✓ | MIT ✓ | `alacritty_terminal` ^0.26 ✓ | `gpui-pre` =0.3.10 ✓ | 1 versie ✓ |
| `bezel-terminal` | 0.2.26, 13 nov 2026 ✓ | MIT ✓ | `alacritty_terminal` ^0.26 ✓ | `bezel-gpui` ^0.3.23 ✓ | 56 versies ✓ |

GPUI Kit 0.9.1 pint `gpui-pre` =0.3.12 ✓. Alleen `gpui_xterm` staat op dezelfde GPUI-versie.

Niets hiervan is gebouwd of getest met Claude Code.

## Proef: Claude Code in een zelfgetekende terminal (2026-11-14)

De code staat in `proef/terminal`: één bestand van ongeveer 550 regels, met `gpui-kit` 0.9.1 en `alacritty_terminal` 0.26.0.

Gemeten op deze Mac (✓):

- **Bouwen:** lukt met Rust 1.100 en alleen de Command Line Tools van Apple. De volledige Xcode is niet nodig. De map `target` is daarna 4,3 GB.
- **Versies:** `gpui-kit` met de optie `test-support` geeft een versieconflict (`taffy`). Dezelfde optie rechtstreeks op `gpui-pre` en `gpui-pre-platform` werkt.
- **Starten:** het venster start `zsh -ilc claude` in de gekozen map.
- **Tekenen:** het startscherm, de keuzeschermen, vet, cursief, gedimd, kleuren, een tabel met lijnen, een codeblok met kleuren, de draaiende regel tijdens het werken en de statusregel komen goed door.
- **Flikkeren:** in tien opnamen tijdens het werken stond geen half getekend scherm.
- **Naad:** in het logo van Claude Code loopt een dunne donkere lijn tussen twee rijen blokjes. De bloktekens uit het lettertype vullen de cel niet precies.

Door mij in het venster geprobeerd op 2026-11-14: het werken in het venster is goed bevonden, en selecteren met de muis en kopiëren met Cmd+C werken. Welke andere onderdelen precies geprobeerd zijn (scrollen, plakken, het venster groter of kleiner maken), is niet vastgelegd.

Niet gebouwd: een pad aanklikken, letters met een accent via een dode toets, en de muis doorgeven aan het programma.

Het venster bewaart zichzelf als afbeelding met `cargo run --features schermafdruk` en `COCKPIT_SHOT=<pad>`. Zo zijn de opnamen gemaakt.

## Bronnen

- **Terminal:** https://crates.io/crates/alacritty_terminal · https://github.com/alacritty/vte/blob/master/src/ansi.rs · https://github.com/Uzaaft/libghostty-rs · https://crates.io/crates/ghostty-vt · https://github.com/Modolet/gpui_xterm · https://github.com/behzade/gpui-libghostty · https://github.com/zortax/gpui-terminal · https://crates.io/crates/ely-terminal · https://crates.io/crates/bezel-terminal
- **GPUI en GPUI Kit:** https://github.com/longbridge/gpui-kit · https://github.com/longbridge/gpui-kit/blob/main/crates/webview/README.md · https://github.com/zed-industries/zed · https://crates.io/crates/gpui · https://crates.io/crates/gpui-pre · https://crates.io/crates/alacritty_terminal
- **Rune:** https://github.com/anomalyco/rune · https://github.com/anomalyco/rune/blob/dev/PLAN.md · https://github.com/anomalyco/rune/blob/dev/crates/rune-kit/README.md · https://github.com/anomalyco/rune/blob/dev/scripts/mac-app.md
- **libghostty:** https://alternativeto.net/news/2026/11/libghostty-1-0-brings-a-stable-c-abi-for-terminals/ · https://runtimewire.com/article/libghostty-1-0-mitchell-hashimoto-stable-c-abi
- **Overige:** https://github.com/slint-ui/slint · https://github.com/emilk/egui · https://github.com/iced-rs/iced · https://github.com/linebender/xilem · https://github.com/lapce/floem
