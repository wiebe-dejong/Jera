# Jera: spec

Spec-fase. Besloten is alleen wat onder "Besloten" staat.

## Doel

Eén Mac-app waarin ik mijn Claude Code-sessies draai en overzie. Die vervangt PyCharm als host voor Claude-terminals.

Erin:
- de sessies per project, met hun status;
- de bestanden van het project;
- de poorten waarop iets luistert, en de agent-browser;
- een git-overzicht;
- de services van het project bij Render, met hun deploystatus.

Niet erin:
- een browser: openen gaat in Safari;
- viewers voor documenten: die openen in een externe app;
- een IDE: voor de debugger blijft PyCharm.

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
| Sessie | Eén regel per sessie, met de titel die Claude Code eraan geeft en de status. Zolang er nog geen titel is, staat er de naam die Claude Code van de map afleidt | 2026-11-14 |
| Linkerkolom | Eén kolom met een schakelaar: Sessies of Bestanden | 2026-11-14 |
| Sessies | De lijst toont alle sessies van de tab, ook als hun terminal niet open staat | 2026-11-14 |
| Bestanden | De mappen en bestanden van het project. Een klik op een bestand opent het in een externe app. Bij elke map kan ik er een sessie starten | 2026-11-14 |
| Terminals in beeld | Standaard één. Een tweede ernaast door een sessie te slepen of met "open ernaast" | 2026-11-14 |
| Sessiestatus | Per sessie is te zien: bezig, wacht op antwoord op een vraag, wacht op toestemming, net klaar, of idle (klaar en doet niets) | 2026-11-14 |
| Schermen bij de start | Vraagt Claude Code bij de start of ik de map vertrouw of een MCP-server wil gebruiken, dan toont de sessie "wacht op toestemming" | 2026-11-14 |
| Net klaar en idle | "Net klaar" wordt "idle" zodra ik de sessie open | 2026-11-14 |
| Status tijdens Bestanden | De knop Sessies krijgt een stip als een sessie op mij wacht | 2026-11-14 |
| Herstart | Na een herstart van Jera komen mijn tabs en sessies terug | 2026-11-14 |
| Sessies van buiten Jera | Een sessie die buiten Jera draait, bijvoorbeeld in PyCharm, staat in de lijst met haar status en een knop Overnemen. Haar terminal is niet te zien | 2026-11-14 |
| Overnemen | Jera stopt de sessie op de oude plek en zet hetzelfde gesprek voort in een eigen terminal. Is de sessie bezig, dan vraagt Jera eerst om bevestiging | 2026-11-14 |
| Afsluiten | Sluit ik Jera terwijl een sessie bezig is, dan waarschuwt Jera eerst | 2026-11-14 |
| Oude sessies | Onderaan de kolom Sessies staat het blok oud, standaard dicht. Open schuift de kop omhoog en zijn er eronder 8 regels te zien; de rest scrolt binnen het blok. Het toont de oude sessies van de tab, alleen met hun titel, gegroepeerd per dag van het laatste bericht onder een klein kopje dat dichtklapt, met het pijltje rechts. Een klik op een oude sessie zet die voort in een nieuwe terminal. Dit verving de stand Eerder in de schakelaar | 2026-11-14 |
| Browser | Geen browser in de app. Openen gaat in Safari | 2026-11-14 |
| agent-browser | Ik zie wanneer er een agent-browser draait, en ik kan die sluiten | 2026-11-14 |
| Bestand openen | In de standaard-app die macOS voor dat bestandstype heeft | 2026-11-14 |
| Paden in de terminal | Een pad in de terminal is aan te klikken en opent het bestand | 2026-11-14 |
| Poorten | Een lijst van alle poorten waarop iets luistert, voor de hele computer. Per poort: stoppen en openen | 2026-11-14 |
| Poorten, welke | Standaard alleen de poorten van een proces dat vanuit een map in mijn thuismap is gestart, ook als Claude het startte. De eigen poorten van Claude Code staan er niet bij. Een knop Alles toont de rest. Het getal op de knop in de tabbalk telt alleen de standaardlijst | 2026-11-14 |
| Startcommando | Jera start elke sessie via mijn eigen shell met `claude`. Het is één vast commando, op één plek aan te passen | 2026-11-14 |
| Paneel voor de hele computer | Poorten en agent-browser staan samen in één paneel, los van de tabs. Een knop rechts in de tabbalk opent het en toont de aantallen | 2026-11-14 |
| Render | Per service van het project: de naam, de link naar de live app, de deploystatus en de commit die live staat. Een klik op een mislukte deploy opent de logs bij Render in Safari. Jera koppelt een service aan een tab via de repo | 2026-11-14 |
| Render, plek | Een smalle regel boven de terminal | 2026-11-14 |
| Eerste versie | Tabs, de sessielijst met status, één terminal, Bestanden, Render en het git-paneel | 2026-11-14 |
| Daarna | In deze volgorde: de oude sessies, het paneel met poorten en agent-browser, de tweede terminal ernaast, Overnemen | 2026-11-14 |
| Git | Een smal git-paneel per repo. Het ontwerp staat als werkend voorbeeld in `git-graaf/`. De logica staat vast | 2026-11-14 |
| Git, plek | Rechts naast de terminal, als paneel dat open en dicht klapt. Het wisselt mee met de tab | 2026-11-14 |
| Thema | Een donker en een licht thema | 2026-11-14 |
| Uiterlijk, grond | Bijna zwart, één vlak, met lijnen van 1 px tussen de panelen. De terminal heeft dezelfde grond. De waarden staan in `stijl/cockpit/keuzes.md` | 2026-11-14 |
| Uiterlijk, tab | Een vlak in de titelbalk, met een streep boven de actieve tab | 2026-11-14 |
| Uiterlijk, status | Een icoon per sessie, zonder woorden: bezig is een groen draaiend boogje, wacht op antwoord een geel vraagteken, wacht op toestemming een rood slot, net klaar een groen vinkje, idle een grijs streepje | 2026-11-14 |
| Uiterlijk, letter | JetBrains Mono overal, 12.5 px | 2026-11-14 |
| Uiterlijk, accent | Groen | 2026-11-14 |
| Uiterlijk, vorm | Rechte hoeken | 2026-11-14 |
| Uiterlijk, dichtheid | Rijen van 26 px | 2026-11-14 |
| Uiterlijk, sessieregel | Het icoon voor de titel. De gekozen sessie is een vlak met een streep in accent links | 2026-11-14 |
| Uiterlijk, schakelaar | Kleine letters, met een streep onder de actieve stand | 2026-11-14 |
| Uiterlijk, Render-regel | Per service een icoon (groen vinkje of rood kruis) en de naam. Een klik op de naam opent de app | 2026-11-14 |
| Uiterlijk, overige onderdelen | Knop, bestandsboom, de regel in het blok oud, het paneel met poorten, de dialoog en het lege scherm zijn gekozen zoals ze in het design system staan | 2026-11-14 |
| Uiterlijk, bestandsiconen | In de bestandsboom staat links van de naam een icoon in kleur: voor bestanden de iconen van GitLab, voor mappen die van Catppuccin. Gekozen uit 25 pakketten, https://claude.ai/artifact/4RPw8CGnNfEKgS9jmvFkLm | 2026-11-14 |
| Iconen, keuze | In de menubalk, onder Jera › Iconen, kies ik tussen Kleur (GitLab en Catppuccin) en Zed (de iconen van Zed, in één kleur). Zed is de standaard. Jera onthoudt de keuze | 2026-11-14 |
| Git, branchnamen | De branchnamen staan boven de lijnen, getrapt: elke naam op een eigen regel | 2026-11-14 |
| Git, stijl | Recht: commits, groepen en eindpunten zijn vierkantjes, een branch splitst af met een rechte hoek, lijnen van 2 px. Geen datumkolom: de dag en het moment van ophalen staan rechts op hun lijn. Gekozen uit drie, https://claude.ai/artifact/LfeunM49x3iSRWZrLbtZLT | 2026-11-14 |
| Git, inklappen | De knop git in de Render-regel klapt het paneel in en uit. Bij het openen van Jera is het ingeklapt | 2026-11-14 |
| Documenten | Geen viewers in de app. Een klik op een document opent het direct in een externe app, zoals Safari, Preview of GridCraft | 2026-11-14 |
| Openen | `+ map` en ⌘O openen het scherm Openen: een zoekveld, dan `open` (met een mapicoon, opent de mapkeuze van macOS) en `clone repository` (met het icoon van git), en daaronder de mappen die ik het laatst in Cockpit opende. Typen filtert, pijltjes en Enter kiezen. Zonder tabs staat Openen open | 2026-11-15 |
| Openen, per regel | Eén kolom: de naam, en klein eronder het pad. Staat het project al als tab open, dan springt kiezen naar die tab | 2026-11-15 |
| Openen, klonen | `clone repository` toont mijn repo's op GitHub, met een zoekveld. Cockpit kloont de gekozen repo in een map die ik elke keer kies, en start er daarna een sessie. Escape gaat terug | 2026-11-15 |
| Nieuw project | Alleen met ⌘N en in de menubalk: ik typ een naam en kies daarna de map waarin het komt. Cockpit maakt de map, doet `git init` en start een sessie. Alleen lokaal | 2026-11-15 |
| Op GitHub | Heeft een repo geen remote, dan toont het git-paneel de eigenaar (een klik wisselt) en de knop Op GitHub. Die maakt een privé repo en zet de code erop | 2026-11-15 |
| Naam | Jera, naar de rune ᛃ voor oogst: een knipoog naar Hidden Harvest. Het teken is het icoon van de app | 2026-11-15 |

Het design system staat in `stijl/design-system/` en op https://claude.ai/artifact/QKTtDToW7LYRivNM1vYG88: de tokens en een voorbeeld per onderdeel. De keuzes zelf staan in `stijl/cockpit/keuzes.md`; het keuzebord staat op https://claude.ai/artifact/6M7pWS1JLMd6jWtFqZkoj3. De feiten bij de stack en de terminal staan in `research/native.md`. De sessiestatus leest Jera uit het sessiebestand van Claude Code; de meting staat in `research/zelf-bouwen.md`.

### Indeling

Schakelaar op Sessies:

```
[ webshop  3 ]  [ cockpit  1 ]  [ + map ]            [ Poorten 4  agent-browser 1 ]
─────────────────────────────────────────────────────────────────────────────────────────────
 [ Sessies ]  Bestanden           │ ● api  api.onrender.com    ● web  web.onrender.com
 webshop/          [+]   │──────────────────────────────────────┬─────────────────
   ● kpi cards            bezig    │                                      │
   ● bug       wacht op antwoord   │   terminal van de gekozen sessie     │   git-paneel
 frontend/                   [+]   │                                      │   (open of dicht)
   ● testing environment  net klaar│                                      │
```

Schakelaar op Bestanden:

```
  Sessies •  [ Bestanden ]         │
 ▸ backend/                  [+]   │
 ▾ frontend/                 [+]   │
   ▸ components/             [+]   │      terminal van de gekozen sessie
     package.json                  │
     README.md                     │
 ▸ docs/                     [+]   │
   render.yaml                     │
```

Schakelaar op Sessies, met het blok oud open en het paneel rechts open:

```
 [ Sessies ]  Bestanden                  │            │ agent-browser  1
 webshop/                 [+]  │            │   default          [Sluiten]
   ● kpi cards                           │            │
                                         │  terminal  │ Poorten  4
─────────────────────────────────────────│            │   3000  node    [Openen] [Stoppen]
 ▾ oud  3                                │            │   8000  python  [Openen] [Stoppen]
 gisteren                            ▾  │            │
   supabase migratie                     │            │
 11 nov                              ▾  │            │
   login flow                            │            │
   tabel sorteren                        │            │
```

De schermen staan ook in het artifact https://claude.ai/artifact/7yXL3y5ykQbPFcZ4GXGXrX (het bord "Gekozen").

## Open

- **Rechts van de terminal:** het git-paneel en het paneel met poorten en agent-browser openen allebei daar. Hoe ze die plek delen.
- **Render:** later misschien: logs en belasting in Jera zelf, en knoppen om te deployen of te herstarten.
- **Terminal:** de proef in `proef/terminal` tekent Claude Code goed (zie `research/native.md`). Ik heb het venster geprobeerd en goed bevonden; selecteren en kopiëren werken. Nog te bouwen: een pad aanklikken, letters met een accent, en bloktekens zonder naad.

## Later

- Meldingen.
