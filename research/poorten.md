# Poorten en agent-browser per sessie (2026-11-15)

Hoe Jera ziet welke sessie een server of agent-browser startte. Gemeten op deze Mac met Claude Code 2.1.325 en agent-browser 0.27.0, met een proefbrowser die vanuit een sessie werd gestart en daarna gesloten.

## Van poort naar sessie

- Een proces dat Claude start, krijgt in zijn omgeving onder meer `CLAUDE_PID` (het pid van de sessie) en `CLAUDE_CODE_SESSION_ID`. Ook gezien: `CLAUDECODE=1`, `CLAUDE_CODE_CHILD_SESSION=1`.
- Die variabelen blijven staan als het proces loskomt van Claude. De daemon van agent-browser had ppid 1 en toch `CLAUDE_PID` en `CLAUDE_CODE_SESSION_ID`. Via de procesboom is de sessie dus niet te vinden, via de omgeving wel.
- `ps -E -ww -o command= -p <pid>` toont de omgeving van een eigen proces.
- De keten: `lsof -nP -iTCP -sTCP:LISTEN` geeft poort en pid → de omgeving geeft `CLAUDE_PID` → `~/.claude/sessions/<CLAUDE_PID>.json` is de sessie. Bestaat dat pid niet meer, dan is het proces een wees.
- Gemeten: een proces dat zijn naam overschrijft (`process.title` in Node, zoals `next-server`) wist ook zijn omgeving; `ps -E` toont dan geen `CLAUDE_PID`. Jera kijkt daarom ook bij de ouders van het proces.
- Gemeten: een app die vanuit een sessie is geopend erft het sessienummer ook. Raycast had `CLAUDE_PID` van een sessie die niet meer draaide en zou dan een wees zijn. Jera telt een proces met een `.app/Contents/` in zijn keten niet als server van een sessie.
- Niet gemeten, wel te verwachten: processen in Docker luisteren via `com.docker.backend` en hebben geen `CLAUDE_PID`. Een agent-browser die twee sessies delen, hoort bij de sessie die hem startte.

## agent-browser

- Per draaiende browser staan in `~/.agent-browser/` de bestanden `<naam>.pid` (pid van de daemon), `<naam>.sock`, `<naam>.engine` (`chrome`), `<naam>.version` en `<naam>.stream`.
- `<naam>.stream` bevat een poort. `agent-browser --session <naam> stream status` toont daar `ws://127.0.0.1:<poort>`, een WebSocket voor het beeld van de browser. Daarmee kan Jera een plaatje tonen. Het formaat van die stream is niet nagekeken.
- Die stream-poort staat in `lsof` als luisterende poort van `agent-browser-darwin-arm64`. Die hoort niet in de lijst met poorten.
- De daemon start Chrome for Testing uit `~/.agent-browser/browsers/` als kindproces, met `--remote-debugging-port=0`.
- `agent-browser --session <naam> close` sluit de browser; daarna zijn de bestanden weg. `agent-browser session list` toont de draaiende sessies.
- `~/.agent-browser/sessions/` bevat opgeslagen cookies per naam, ook van browsers die niet draaien. Dat zegt niets over wat er draait.

## Wat er op 2026-11-15 luisterde

- 3× Claude Code zelf (procesnaam `2.1.325`, op 127.0.0.1);
- ControlCenter op 5000 en 7000 (AirPlay);
- Raycast (`node`, 7265, werkmap `/`);
- rapportd.

Geen van die processen had een sessie gestart.
