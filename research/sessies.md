# Claude Code-sessies van buiten gelezen (2026-11-14)

Hoe Jera ziet wat een sessie doet. Gemeten op deze Mac; de lijst met manieren komt uit het onderzoek van 2026-11-04.

## Status en sessiebestand

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
  - `name` in het sessiebestand is afgeleid van de map, bijvoorbeeld `jera-70`. De titel in het sessiebestand van de transcriptie (`ai-title`) is een andere naam, bijvoorbeeld "Rust native app maken";
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

## Koppeling met Claude Code

1. **Hooks** (gedocumenteerd, laag risico):
   - `type: "http"`-hooks voor `Notification`, `Stop` en `SessionStart`;
   - geef elke PTY een `JERA_TERMINAL_ID` mee en stuur die als header via `allowedEnvVars`, zodat de app weet welke terminal aandacht nodig heeft.
2. **Eigen MCP-server** (gedocumenteerd, laag risico):
   - tools als `open_file`, `open_url`, `screenshot_tab`, `console_logs`, `restart_server` en `tail_logs`;
   - aanmelden met `claude mcp add --transport http`.
3. **IDE-protocol** (werkt, maar niet officieel):
   - een WebSocket-MCP op 127.0.0.1 met een lockfile in `~/.claude/ide/<poort>.lock`;
   - geeft `openDiff`, `openFile`, `getDiagnostics` en `selection_changed`;
   - de referentie is claudecode.nvim (`PROTOCOL.md`);
   - pas in v3 en het kan breken bij updates.
4. **Agent SDK:** met een eigen product mag je geen claude.ai-login aanbieden, dus je betaalt API-tarieven. Je bouwt dan ook zelf een chat-UI. De echte CLI in een PTY houdt je abonnement en alle functies.

## Bronnen

- **Claude Code:** https://code.claude.com/docs/en/desktop · https://code.claude.com/docs/en/terminal-config · https://code.claude.com/docs/en/hooks · https://code.claude.com/docs/en/agent-view · https://code.claude.com/docs/en/ide-integrations · https://code.claude.com/docs/en/agent-sdk/overview · https://github.com/coder/claudecode.nvim/blob/main/PROTOCOL.md
