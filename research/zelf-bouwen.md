# Zelf bouwen (2026-11-04)

## App-bases

| Optie | Feiten |
|---|---|
| Electron (stabiel: 45.x) | WebContentsView is gewoon Chromium, dus je hebt echte DevTools en CDP, en agent-browser en Playwright kunnen de browser besturen. node-pty en xterm.js zijn bewezen (VS Code, Wave en Superset gebruiken ze). Alles is TypeScript en React, met hot reload. Kost ~150–400 MB, plus ~50–150 MB per browsertab. |
| Tauri v2 | Meerdere webviews in één venster is nog experimenteel. Er zijn open macOS-bugs met child-webviews. WKWebView heeft geen CDP. DevTools in release-builds gebruiken een private Apple-API. De backend is Rust. |
| Native Swift | Het zuinigst met geheugen. Documentweergave via PDFKit en Quick Look. WKWebView heeft geen CDP. libghostty: de emulatiekern heeft sinds 6 nov 2026 een stabiele API, zie `native.md`. |
| Lokale webapp | Sites blokkeren iframes, je kunt in een iframe geen DevTools openen, en de browser vangt ⌘W, ⌘T en ⌘N zelf af. |
| Wave forken | Apache-2.0, Electron, Go en TypeScript. Een grote codebase, waarvan de helft Go is. Een fork moet upstream bijhouden. |

## Onderdelen

### Terminal

- **xterm.js 6 met de WebGL-addon en node-pty (MIT).**
  - Versie 6 ondersteunt synchronized output (DEC 2026), wat Claude Code nodig heeft om niet te flikkeren.
  - Flikkert het toch, zet dan `CLAUDE_CODE_NO_FLICKER=1` of `CLAUDE_CODE_FORCE_SYNC_OUTPUT=1`.
- **Shift+Enter** moet `\n` sturen.
- **WebGL-limiet:** Chromium staat ongeveer 16 WebGL-contexten tegelijk toe. Haal de WebGL-addon weg bij terminals die niet zichtbaar zijn en vang het verlies van een context op.
- **ghostty-web** (MIT, van Coder) heeft dezelfde API als xterm.js.
- **Zien of Claude op je wacht:**
  - HTTP-hooks;
  - `claude agents --json --all`;
  - OSC 7501 (Claude Code 2.1.295 en nieuwer);
  - `~/.claude/sessions/*.json`.
- **Gemeten op deze Mac op 2026-11-14** (Claude Code 2.1.325), met een proefsessie die een vraag stelde en om toestemming vroeg:
  - `~/.claude/sessions/<pid>.json` bevat per sessie onder meer `cwd`, `name`, `status` en `statusUpdatedAt`;
  - `status` is `busy` terwijl Claude werkt en `idle` als Claude klaar is. Ook gezien: `shell`;
  - bij een vraag (de tool `AskUserQuestion`) staat er `status: "waiting"` met `waitingFor: "input needed"`;
  - bij een verzoek om toestemming staat er `status: "waiting"` met `waitingFor: "permission prompt"`;
  - het bestand verandert binnen een seconde na de gebeurtenis;
  - `claude agents --json --all` geeft in beide gevallen `status: "waiting"`. Of de reden erbij staat, is niet nagekeken;
  - `name` in het sessiebestand is afgeleid van de map, bijvoorbeeld `cockpit-70`. De titel in het sessiebestand van de transcriptie (`ai-title`) is een andere naam, bijvoorbeeld "Rust native app maken";
  - het formaat van dit bestand staat niet in de documentatie van Claude Code.
- **Hooks, gemeten in dezelfde proef:**
  - `PreToolUse` met `tool_name: "AskUserQuestion"` komt direct bij een vraag;
  - `PermissionRequest` komt direct bij een verzoek om toestemming, en ook bij een vraag. `tool_name` maakt het verschil;
  - `Notification` met `permission_prompt` komt ongeveer zes seconden later, bij allebei, met dezelfde tekst;
  - `Stop` komt als Claude klaar is. `Notification` met `idle_prompt` komt zestig seconden daarna.
- **Schermen bij de start:** in een nieuwe map vraagt Claude Code eerst of ik de map vertrouw; de standaardkeuze is "No, exit". In een map met een nieuwe MCP-server vraagt het of die gebruikt mag worden. Tijdens het vertrouwensscherm bestaat het sessiebestand nog niet.
  - een oude sessie staat in `~/.claude/projects/<pad>/<sessie-id>.jsonl`. Dat bestand bevat regels van het type `ai-title` met de titel van de sessie. `claude --resume <sessie-id>` start de sessie opnieuw.
  - volgens `claude --help` (2.1.325) start `--resume` een kopie als de sessie al ergens draait. `--fork-session` geeft bij het hervatten een nieuw sessie-id. Overnemen van een draaiende sessie is niet gemeten.
- **Gewone OSC-codes:** 9 en 777 (meldingen), 9;4 (voortgang), 7 (werkmap), 133 (prompt).
- **tmux** haalt de synchronized output weg.

### Documenten

| Type | Optie |
|---|---|
| PDF | De ingebouwde viewer van Chromium, geen code nodig |
| Markdown | react-markdown met remark-gfm en shiki |
| CSV | papaparse met een gevirtualiseerde tabel |
| xlsx | `win.previewFile()`, dat macOS Quick Look opent (gratis en lijkt precies op Excel). De gratis SheetJS geeft alleen waarden, geen opmaak. |

### Git

- **De git-CLI aanroepen** (`--porcelain=v2 -z`, of via simple-git). Worktrees, hooks en inloggegevens werken dan net als bij Claude.
- **isomorphic-git** heeft geen SSH. **nodegit** is stilgevallen sinds juli 2020.
- **Diffs:** @codemirror/merge.
- **Hunks stagen:** een patch maken en `git apply --cached` draaien.

### Servers

- **Configformaat:** `.claude/launch.json`, hetzelfde als Claude Desktop.
- **Starten:** elke server in een eigen PTY en process group.
- **Stoppen:**
  1. SIGTERM naar de hele process group.
  2. Na een time-out SIGKILL.
  3. Ook de procesboom aflopen, want `npm`, `uvicorn --reload` en turbo starten eigen groepen.
- **Poorten:** `lsof -nP -iTCP -sTCP:LISTEN`, plus de werkmap van elk proces om het aan een project te koppelen.
- **Gemeten op deze Mac op 2026-11-14:** er luisteren 25 poorten. Per proces, met de werkmap:
  - 1 dev-server: `node` op 3055, werkmap in een projectmap;
  - 4 van Claude Code zelf, werkmap in een projectmap;
  - 5 van agent-browser en 5 van zijn Chrome, werkmap in een tijdelijke map onder `/private/tmp`;
  - 5 van PyCharm en 1 van `cef_server` van PyCharm, werkmap `/` of in `/Applications`;
  - 3 van macOS (ControlCenter op 5000 en 7000, `rapportd`), werkmap `/`;
  - 1 van Raycast (`node` op 7265), werkmap `/`.
- **Startcommando, gemeten op 2026-11-14:** `claude` is in `~/.zshrc` een functie die eerst `.env.local` uit de huidige map laadt en daarna Claude Code start. Het programma `claude` rechtstreeks starten slaat die stap over.
- **Bewaken:**
  - health via HTTP;
  - bij een crash herstarten met backoff;
  - logs in een ringbuffer en in een bestand.
- **Opruimen:** een pidfile bijhouden en bij het starten van de app achtergebleven servers opruimen.

### Browser

- **Tabs:** één WebContentsView per tab, met een eigen opslagpartitie per project. Zo blijft per project het juiste GitHub-account ingelogd.
- **DevTools:** via `openDevTools` of `setDevToolsWebContents`.
- **CDP:**
  - zet het aan met `app.commandLine.appendSwitch('remote-debugging-port', …)`;
  - daarna werkt `agent-browser connect <poort>`.
- **Risico:** CDP geeft toegang tot alle webContents, dus ook tot de UI van de app zelf. Filter die via een kleine proxy, of bied browsertools aan via de eigen MCP-server.

### Deploys

- **Render:** `GET /v1/logs`, een WebSocket op `/v1/logs/subscribe`, en de endpoints voor services en deploys.
- **Supabase:** `GET /v1/projects/{ref}/health` en de log-endpoints (maximaal 30 verzoeken per minuut).
- **Tokens:** opslaan via Electron `safeStorage`, in de Keychain.

## Koppeling met Claude Code

1. **Hooks** (gedocumenteerd, laag risico):
   - `type: "http"`-hooks voor `Notification`, `Stop` en `SessionStart`;
   - geef elke PTY een `COCKPIT_TERMINAL_ID` mee en stuur die als header via `allowedEnvVars`, zodat de app weet welke terminal aandacht nodig heeft.
2. **Eigen MCP-server** (gedocumenteerd, laag risico):
   - tools als `open_file`, `open_url`, `screenshot_tab`, `console_logs`, `restart_server` en `tail_logs`;
   - aanmelden met `claude mcp add --transport http`.
3. **IDE-protocol** (werkt, maar niet officieel):
   - een WebSocket-MCP op 127.0.0.1 met een lockfile in `~/.claude/ide/<poort>.lock`;
   - geeft `openDiff`, `openFile`, `getDiagnostics` en `selection_changed`;
   - de referentie is claudecode.nvim (`PROTOCOL.md`);
   - pas in v3 en het kan breken bij updates.
4. **Agent SDK:** met een eigen product mag je geen claude.ai-login aanbieden, dus je betaalt API-tarieven. Je bouwt dan ook zelf een chat-UI. De echte CLI in een PTY houdt je abonnement en alle functies.

## IDE

- **Claude doet al:** bouwen, draaien, tests, lint, type-checks en refactors.
- **PyCharm heeft:** de debugger, LSP en refactors met de hand.

## Aanpasbaarheid

- **Eén configbestand** (TOML of JSON, gecontroleerd met zod) dat direct herlaadt. Bij een fout in de config een toast, geen crash.
- **Commando's zijn de API.** Elke actie staat in de command palette, en sneltoetsen koppelen een toets aan een actie-id.
- **Scriptcommando's** zoals in Raycast: een script met een header in een map leggen en het staat in de palette.

## Wat moeilijk is (groot risico eerst)

1. **De app ontwikkelen terwijl je erin werkt.** Een herstart sluit je Claude-sessies en servers af. Gebruik een stabiele build en een aparte dev-build, en zet later de PTY's in een apart pty-host-proces.
2. **Processen netjes afsluiten,** zodat er geen servers achterblijven.
3. **De terminal moet Claudes interface goed tonen:** flikkeren, scrollback, IME, Option als Meta, grote pastes, de WebGL-limiet.
4. **WebContentsView ligt over de DOM heen,** dus menu's en dialogen verdwijnen eronder. Verberg of verklein de view bij elke overlay, en houd de afmetingen gelijk met een ResizeObserver.
5. **Focus en sneltoetsen** verdelen over terminal, webviews en de app.
6. **Onderhoud:** Electron brengt elke 8 weken een hoofdversie uit en ondersteunt alleen de laatste 3, en node-pty moet na een upgrade opnieuw gebouwd worden. Zet versies vast.
7. **Bestanden volgen:** gebruik @parcel/watcher en negeer `node_modules` en `.venv`.
8. **Ondertekenen hoeft niet** voor eigen gebruik: electron-builder doet ad-hoc ondertekening. Kanttekening: macOS kan na elke build opnieuw om rechten vragen. Een zelf aangemaakt certificaat lost dat op.

## Bronnen

- **Claude Code:** https://code.claude.com/docs/en/desktop · https://code.claude.com/docs/en/terminal-config · https://code.claude.com/docs/en/hooks · https://code.claude.com/docs/en/agent-view · https://code.claude.com/docs/en/ide-integrations · https://code.claude.com/docs/en/agent-sdk/overview · https://github.com/coder/claudecode.nvim/blob/main/PROTOCOL.md
- **Bestaande tools:** https://github.com/manaflow-ai/cmux · https://github.com/wavetermdev/waveterm · https://github.com/superset-sh/superset
- **Electron:** https://releases.electronjs.org/ · https://www.electronjs.org/docs/latest/api/web-contents-view · https://www.electronjs.org/docs/latest/api/base-window · https://www.electronjs.org/docs/latest/tutorial/code-signing · https://www.electron.build/docs/mac · https://github.com/microsoft/playwright/issues/39008 · https://agent-browser.dev/cdp-mode
- **Tauri:** https://github.com/Fledge-Software/Syrinx-app/wiki/Background-01-shell-and-engine · https://v2.tauri.app/develop/debug/ · https://github.com/bindreams/hole/issues/129
- **Terminal:** https://github.com/xtermjs/xterm.js/releases · https://github.com/coder/ghostty-web · https://github.com/microsoft/node-pty · https://mitchellh.com/writing/libghostty-is-coming
- **Deploys:** https://api-docs.render.com/reference/list-logs · https://supabase.com/docs/reference/api
- **Documenten:** https://github.com/dream-num/univer
