# Bestaande tools (2026-11-04)

Er is geen product dat alle vijf onderdelen goed doet:
- terminal;
- documenten;
- git;
- servers;
- browser.

Daarbovenop:
- **Geen enkel product toont xlsx.**
- **Geen enkel product toont de status van Render of Supabase.**

Vier tools komen het dichtst bij, met elk vier van de vijf onderdelen: Orca, cmux, Wave Terminal en Paradev. Paradev heette vroeger Superset en is rond 26 okt 2026 hernoemd.

Correctie op het oorspronkelijke onderzoek: dat ging ervan uit dat ik meestal één sessie draai. Gemeten zijn het er vaak 7 tegelijk (zie `werkwijze.md`). Tools die gebouwd zijn voor veel agents tegelijk passen dus beter dan dat onderzoek dacht.

## Vergelijking

Legenda: ✓ = goed, ~ = deels, ✗ = niet.
- "CC TUI": Claude Code draait als echte terminal-app.
- "Chat-UI": de tool wikkelt Claude in een eigen chatvenster.

| Product | Terminal | Docs | Git | Servers | Browser | Aanpasbaar | Licentie, stack | Prijs |
|---|---|---|---|---|---|---|---|---|
| **Orca** [1–4] | ✓ CC TUI | ~ md, pdf, afbeeldingen, html | ✓ diff, commit, PR | ~ servers draaien als terminal | ✓ agents kunnen hem besturen | middel | MIT · Electron, React, Tailwind | gratis |
| **cmux** [5–9] | ✓ libghostty | ~ md | ~ diff, branch, PR | ~ toont poorten | ✓ scriptbaar | **hoog**: `cmux.json`, socket-API | GPL-3.0 · Swift | gratis (+Pro) |
| **Wave Terminal** [10–15] | ✓ CC TUI | **✓** md, pdf, csv, afbeeldingen, Monaco | ✗ | ~ `cmd`-widgets | ✓ web-widget | **hoog**: `widgets.json` | Apache-2.0 · Electron, Go, React | gratis |
| **Paradev** [16–18] | ✓ | ~ md, afbeeldingen | ✓ | ✓~ poortdetectie | ✓ DevTools, CDP | middel | ELv2 · Electron, React | gratis, Pro betaald |
| **Claude Desktop (Code-tab)** [19–20] | ✓ chat-UI | ~ pdf, html | ✓ diff-commentaar, PR | ✓ `.claude/launch.json` | ✓ | laag | gesloten | Claude-abonnement |
| **Conductor** [21] | ✓ chat-UI | ? | ✓ | ~ `conductor.json` | ~ | laag | gesloten | gratis |
| **Solo** [22] | ✓ | ~ | ✗ | **✓✓** `solo.yml`, herstart, MCP | ✗ | middel | gesloten · Tauri | gratis tot 4 projecten, $99/jaar |
| **Nimbalyst** [23] | ✓ chat-UI | ✓ md, csv | ✓ | ✗ | ✗ | hoog: extensie-API | MIT · Electron | gratis |
| **Jean** [24] | ✓ chat-UI | ~ | ✓ | ✗ | ✓ | middel | Apache-2.0 · Tauri v2, React 19 | gratis |
| **VS Code** [25] | ✓ | ✓ via extensies | ✓ | ~ tasks | ✓ ingebouwd | maximaal | MIT | gratis |

**Ook bekeken:**
- **Warp:** AGPL, Rust, geen browser [26].
- **Emdash:** geen viewer en geen browser [27].
- **Vibe Kanban:** het bedrijf stopte in april 2026 [28].
- **Opcode:** stilgevallen [29].
- **MacCode:** klein project [30].
- **Ghostty 1.3** [31] en **iTerm2 3.6** [32]: terminals, geen cockpit.
- **Cursor 3** [33], **Antigravity 2.0** [34], **Zed 1.0** [35]: editors.
- **dekit**, de opvolger van mprocs [36].
- **GitButler**: licentie met non-compete [37].

## De drie die het dichtst bij komen

### Orca

- **Sterk in:** de browser die agents kunnen besturen, en viewers voor pdf en md.
- **Mist:**
  - csv en xlsx;
  - een echte serverbeheerder (geen herstart, geen logpaneel, geen healthcheck);
  - aanpasbaarheid, die vooral uit instellingen bestaat.
- **Verder:** de opzet is één worktree per taak, zonder aparte poorten of database per taak. Telemetrie staat standaard aan en de app gebruikt veel RAM.

### cmux

- **Sterk in:** aanpasbaarheid. Per project start een `.cmux/cmux.json` een complete werkruimte op.
- **Mist:**
  - pdf, csv en afbeeldingen;
  - een commit-UI;
  - servers herstarten.
- **Licentie:** GPL, en alleen voor Mac.

### Wave Terminal

- **Sterk in:** documenten (de beste viewer van deze groep) en widgets die je via config instelt.
- **Mist:**
  - een git-UI;
  - automatisch herstarten van servers;
  - een gedocumenteerde manier waarop Claude Code de browser kan besturen.
- **Onderhoud:** geen stabiele release sinds april 2026.

## Licentie en forkbaarheid

- **Orca:** MIT. Electron, React en Tailwind. Er komt bijna elke dag een release uit, dus een fork blijft achterlopen.
- **Wave:** Apache-2.0. Uitbreiden gaat via het blok- en widgetsysteem. Half Go.
- **Jean:** Apache-2.0, Tauri. Licht, klein, met een chat-UI.
- **Nimbalyst:** MIT, met een extensie-API.
- **cmux:** GPL. Uitbreiden kan via de config en de socket-API.
- **Paradev:** ELv2.
- **Warp:** AGPL.
- **GitButler:** licentie met non-compete.
- De gesloten apps hebben geen open bron.

## Wat niemand doet

- Servers beheren die Claude zelf heeft gestart.
- Een browser met een eigen sessie per project.
- xlsx tonen.
- De deploystatus van Render en Supabase tonen.
- Aanpasbaarheid en volledigheid combineren: tools die goed aan te passen zijn, missen onderdelen, en complete tools zijn gesloten.

## Bronnen

[1] https://github.com/stablyai/orca · [2] https://www.onorca.dev/docs · [3] https://www.oflight.co.jp/en/columns/orca-agent-development-environment-parallel-worktrees-2026 · [4] https://betterstack.com/community/guides/ai/orca-ai/ · [5] https://github.com/manaflow-ai/cmux · [6] https://cmux.com/ · [7] https://cmux.com/docs/configuration · [8] https://cmux.com/docs/custom-commands · [9] https://cmux.com/docs/changelog · [10] https://github.com/wavetermdev/waveterm · [11] https://github.com/wavetermdev/waveterm/releases · [12] https://docs.waveterm.dev/customwidgets · [13] https://raw.githubusercontent.com/wavetermdev/waveterm/main/docs/docs/tabs.mdx · [14] https://raw.githubusercontent.com/wavetermdev/waveterm/main/ROADMAP.md · [15] https://docs.waveterm.dev/claude-code · [16] https://github.com/ParadevAI/Paradev · [17] https://docs.paradev.com/browser · [18] https://magica.com/news/paradev-seed-funding-coding-agent-workspace · [19] https://code.claude.com/docs/en/desktop · [20] https://claude.com/blog/preview-review-and-merge-with-claude-code · [21] https://www.conductor.build/docs/ · [22] https://soloterm.com · [23] https://github.com/nimbalyst/nimbalyst · [24] https://github.com/coollabsio/jean · [25] https://github.blog/changelog/2026-07-08-github-copilot-in-visual-studio-code-june-2026-releases/ · [26] https://www.helpnetsecurity.com/2026/04/30/warp-open-source-client/ · [27] https://github.com/generalaction/emdash · [28] https://vibekanban.com/blog/shutdown · [29] https://crystl.dev/compare/claude-code-guis/ · [30] https://github.com/samuelfaj/MacCode · [31] https://ghostty.org/docs/install/release-notes/1-3-0 · [32] https://iterm2.com/documentation-web.html · [33] https://www.datacamp.com/blog/cursor-3 · [34] https://www.analyticsvidhya.com/blog/2026/05/google-antigravity-2-0/ · [35] https://vibecodehq.pages.dev/blog/zed-1-0-agent-client-protocol-external-agents-may-2026 · [36] https://github.com/pvolok/dekit · [37] https://blog.gitbutler.com/gitbutler-is-now-fair-source
