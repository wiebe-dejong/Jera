# Agents van een sessie (2026-11-14)

Hoe Jera kan zien welke agents een sessie heeft en wat ze doen.

**Legenda:**
- ✓ = gemeten op 2026-11-14 met Claude Code 2.1.325. Zes proefagents op Haiku, in een sessie met `bypassPermissions`. Een script las `~/.claude` elke 0,5 s.
- (g) = gezien in bestaande bestanden, niet zelf uitgelokt.
- Het formaat staat niet in de documentatie en kan bij elke versie veranderen.

## Soorten

| Soort | Ontstaat door | Bij mij |
|---|---|---|
| Subagent | De Agent-tool, met of zonder naam | 219 in een week, in 24 sessies (g) |
| Teamlid | De Agent-tool met een naam, in een team | 17 in één sessie van `vydar-site` (g) |
| Achtergrondtaak | `jobs/`, een sessie zonder terminal | 3, de laatste uit juli (g). Niet gemeten |

Een naam maakt nog geen teamlid (2026-11-15, (g)): drie agents in een sessie van Jera hebben `name` maar geen `teamName` en geen `color`, wel `finished: true`, en ontbreken in elk `config.json`. Herken een teamlid daarom aan `teamName` (en `taskKind: in_process_teammate`), niet aan `name`.

Elke sessie heeft vanaf de start een eigen team, `session-<eerste 8 tekens van het sessie-id>`, met alleen `team-lead` erin ✓.

## Waar het staat

| Bestand | Wat erin staat |
|---|---|
| `projects/<map>/<sessie-id>/subagents/agent-<id>.meta.json` | Per agent: `description`, `agentType`, `model`. Een subagent krijgt `finished: true` als hij klaar is. Een teamlid heeft daarnaast `name`, `color` en `teamName` ✓ |
| `projects/<map>/<sessie-id>/subagents/agent-<id>.jsonl` | Het gesprek van de agent, in hetzelfde formaat als dat van een sessie ✓ |
| `teams/session-<id8>/config.json` | `leadProcess.pid` en `members`: de teamleden die nog bestaan, met naam, kleur, model en opdracht ✓ |
| `teams/session-<id8>/inboxes/<naam>.json` | Postvak per teamlid en voor `team-lead`. Een bericht staat er minder dan 0,5 s in, daarna is het bestand `[]` ✓ |

## De stand per agent

| Stand | Subagent | Teamlid |
|---|---|---|
| Bezig | Geen `finished` en niet gestopt ✓ | De laatste regel van zijn gesprek is geen assistant-regel met `end_turn` ✓ |
| Klaar | `finished: true` in `meta.json`, ongeveer 1 s na het laatste toolresultaat ✓. Alleen `finished` telt, zie hieronder | De laatste regel is assistant met `end_turn`. Er komt een `idle_notification` in `inboxes/team-lead.json`, maar die is binnen 0,5 s weer weg ✓ |
| Weer aan het werk | Komt niet voor | Een bericht in `inboxes/<naam>.json`, daarna een nieuwe user-regel in zijn gesprek ✓ |
| Gestopt | Nooit `finished`. De laatste regel is `[Request interrupted by user for tool use]` ✓ | Verdwijnt uit `members` in `config.json`. Zijn gesprek en `meta.json` blijven staan en zien eruit als klaar ✓ |

Een subagent kan `end_turn` geven terwijl hij nog werkt (2026-11-15, Claude Code 2.1.325, in Jera) ✓:
- "Slaapt veertig seconden" startte zijn `sleep 40` op de achtergrond, gaf om 23:35:47 `end_turn`, en leverde pas om 23:36:27 zijn rapport in. Claude Code toonde hem intussen als *waiting on its own background work*.
- Zijn rapport gaat met de tool `SubagentHandback`. De laatste regel van zijn gesprek is dan het toolresultaat ("Report delivered"), geen `end_turn`.
- Lees een subagent dus alleen als klaar met `finished`. Voor een teamlid blijft `end_turn` de enige aanwijzing.

Een teamlid krijgt nooit `finished`. Dat verklaart een deel van de 74 subagents zonder `finished` van de afgelopen week; de rest is gestopt of nog bezig.

## Wat een agent doet

- De laatste assistant-regel in zijn gesprek met een `tool_use` geeft de tool en zijn invoer, zoals `Bash` met `description` "Build Jera release" ✓ (2026-11-15, een subagent die bezig was).
- Tussen de regels van een gesprek staan ook regels van het type `api-request`, `api-request-blob` en `attachment` (g, in ruim 900 gesprekken). Een gesprek dat klaar is, eindigt wel steeds op een assistant- of user-regel.

## De sessie zelf

- Zolang een subagent of teamlid werkt, blijft het sessiebestand op `busy`, ook als de beurt van de sessie zelf voorbij is ✓ (twee keer 40 s).
- De sessie gaat pas naar `idle` als alles klaar is, op dezelfde milliseconde als het laatste teamlid ✓.
- Een teamlid dat klaar is maar nog bestaat, houdt de sessie niet op `busy` ✓.

De status die Jera nu toont, klopt dus al met agents erbij. Agents geven extra inzicht: wie er werkt en waaraan.

## Koppelen aan een sessie in Jera

- `sessions/<pid>.json` geeft het `sessionId`. Daaruit volgen de map `subagents/` en de teamnaam `session-<id8>`.
- Een teammap blijft soms staan nadat de sessie is gestopt (g): `teams/eurekaweek/` is van maart. Controleer of `leadProcess.pid` nog draait.

## Open

- **Toestemming of een vraag van een agent.** Niet te meten in `bypassPermissions`. Daarvoor is een proef nodig in een sessie in de standaardmodus.
- **Subagent in de voorgrond** (`requestShape: foreground`): niet gemeten.
- **Achtergrondtaken** (`jobs/`): niet gemeten.
- **Bezorging.** De klaar-meldingen van een teamlid kwamen twee keer pas minuten later in de sessie binnen, bij de volgende melding ✓. Lees de stand daarom uit het gesprek van het teamlid, niet uit het postvak.
