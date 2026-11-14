# Cockpit: spec

Spec-fase. Besloten is alleen wat onder "Besloten" staat.

## Doel

Eén Mac-app voor mijn dagelijkse ontwikkelwerk. Die vervangt:
- PyCharm, als host voor Claude-terminals;
- Safari en Chrome, voor localhost en de live apps;
- Preview en Excel, om te bekijken;
- de dashboards van Render en Supabase.

Geen IDE: voor de debugger blijft PyCharm.

## Gemeten werkwijze

Uit `research/werkwijze.md`:
- Claude doet het werk: start zelf de dev-servers, maakt ~77% van de commits, pusht, en regelt deploys en database via MCP.
- Ik draai vaak 7 sessies tegelijk.
- Ik controleer in Safari.
- Servers en poorten raken een rommel: ~25 poorten, honderden keren `kill` en `lsof`.

## Besloten

| Onderwerp | Keuze | Datum |
|---|---|---|
| Stack | GPUI met GPUI Kit (Rust) | 2026-11-14 |
| Terminal, kern | `alacritty_terminal` | 2026-11-14 |
| Terminal, tekenen in GPUI | Zelf bouwen | 2026-11-14 |
| Sessie starten | Ik kies een map en start daar een Claude-sessie | 2026-11-14 |
| Tab | Eén tab per git-repo. Een map zonder git is zijn eigen tab. Een worktree valt onder zijn repo | 2026-11-14 |
| Groep | Binnen een tab staan de sessies gegroepeerd per map: de hoofdmap of een submap | 2026-11-14 |
| Sessie | Eén regel per sessie, met de naam die Claude Code eraan geeft en de status | 2026-11-14 |
| Linkerkolom | Eén kolom met een schakelaar: Sessies, Bestanden of Eerder | 2026-11-14 |
| Sessies | De lijst toont alle sessies van de tab, ook als hun terminal niet open staat | 2026-11-14 |
| Bestanden | De mappen en bestanden van het project. Een klik op een bestand opent het in een externe app. Bij elke map kan ik er een sessie starten | 2026-11-14 |
| Terminals in beeld | Standaard één. Een tweede ernaast door een sessie te slepen of met "open ernaast" | 2026-11-14 |
| Sessiestatus | Per sessie is te zien: bezig, wacht op antwoord op een vraag, wacht op toestemming, net klaar, of idle (klaar en doet niets) | 2026-11-14 |
| Net klaar en idle | "Net klaar" wordt "idle" zodra ik de sessie open | 2026-11-14 |
| Status tijdens Bestanden | De knop Sessies krijgt een stip als een sessie op mij wacht | 2026-11-14 |
| Herstart | Na een herstart van Cockpit komen mijn tabs en sessies terug | 2026-11-14 |
| Oude sessies | De stand Eerder toont de oude sessies van de tab, gegroepeerd per map. Daaruit start ik er een opnieuw | 2026-11-14 |
| Browser | Geen browser in de app. Openen gaat in Safari | 2026-11-14 |
| agent-browser | Ik zie wanneer er een agent-browser draait, en ik kan die sluiten | 2026-11-14 |
| Bestand openen | In de standaard-app die macOS voor dat bestandstype heeft | 2026-11-14 |
| Paden in de terminal | Een pad in de terminal is aan te klikken en opent het bestand | 2026-11-14 |
| Poorten | Een lijst van alle poorten waarop iets luistert, voor de hele computer. Per poort: stoppen en openen | 2026-11-14 |
| Paneel voor de hele computer | Poorten en agent-browser staan samen in één paneel, los van de tabs. Een knop rechts in de tabbalk opent het en toont de aantallen | 2026-11-14 |
| Git | Een duidelijk git-overzicht. De invulling volgt apart | 2026-11-14 |
| Documenten | Geen viewers in de app. Een klik op een document opent het direct in een externe app, zoals Safari, Preview of GridCraft | 2026-11-14 |

De feiten bij de stack en de terminal staan in `research/native.md`.

### Indeling

Schakelaar op Sessies:

```
[ webshop  3 ]  [ cockpit  1 ]  [ + map ]     [ Poorten 4  agent-browser 1 ]
──────────────────────────────────────────────────────────────────────────
 [ Sessies ]  Bestanden  Eerder    │
 webshop/          [+]   │
   ● kpi cards            bezig    │
   ● bug       wacht op antwoord   │      terminal van de gekozen sessie
 frontend/                   [+]   │
   ● testing environment  net klaar│
```

Schakelaar op Bestanden:

```
  Sessies •  [ Bestanden ]  Eerder │
 ▸ backend/                  [+]   │
 ▾ frontend/                 [+]   │
   ▸ components/             [+]   │      terminal van de gekozen sessie
     package.json                  │
     README.md                     │
 ▸ docs/                     [+]   │
   render.yaml                     │
```

Schakelaar op Eerder, met het paneel open:

```
  Sessies •  Bestanden  [ Eerder ]       │            │ agent-browser  1
 webshop/                      │            │   default          [Sluiten]
   supabase migratie  gisteren  [Hervat] │            │
   login flow         11 nov    [Hervat] │  terminal  │ Poorten  4
 frontend/                               │            │   3000  node    [Openen] [Stoppen]
   tabel sorteren     9 nov     [Hervat] │            │   8000  python  [Openen] [Stoppen]
```

De schermen staan ook in het artifact https://claude.ai/artifact/7yXL3y5ykQbPFcZ4GXGXrX (het bord "Gekozen").

## Open

- **Git:** de invulling van het overzicht.
- **Poorten:** of processen van macOS zelf in de lijst staan.
- **Startcommando:** waarmee een nieuwe sessie start.
- **Uiterlijk:** kleuren, lettertypen, dichtheid en hoe de vijf statussen eruitzien.
- **Meldingen:** later.
- **Terminal:** `alacritty_terminal` negeert de codes voor werkmap, meldingen en voortgang (OSC 7, 9, 777, 9;4, 133). Niet getest met Claude Code.
