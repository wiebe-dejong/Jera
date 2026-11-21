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
- viewers voor andere documenten dan die in Documenten: die openen in een externe app;
- een IDE: voor de debugger blijft PyCharm.

## Gemeten werkwijze

Gemeten op 2026-11-04:
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
| Bestanden | De mappen en bestanden van het project. Een klik op een bestand opent het in Documenten, of in een externe app als Documenten het niet toont. Bij elke map kan ik er een sessie starten | 2026-11-15 |
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
| Bestand openen | Markdown, csv, tsv, xlsx (en xls, ods), json, png, jpg, gif, webp, svg en pdf in Documenten; al het andere, ook html, in de standaard-app die macOS voor dat bestandstype heeft | 2026-11-15 |
| Documenten | Eén apart venster met een tab per bestand. Een bestand dat al open is, krijgt geen tweede tab. Verandert het bestand op schijf, dan toont de tab de nieuwe versie en blijft de plek. Rechts in de titelbalk `in app openen`, ook in het menu van een twee-vingerklik in Bestanden; bij markdown en json staat het op geen van beide plekken. ⌘W sluit een tab, de laatste sluit het venster | 2026-11-15 |
| Documenten, per soort | Markdown opgemaakt, koppen vet en even groot als de tekst. Tabellen met rijnummers en alleen de rijen in beeld getekend; een werkboek met letters boven de kolommen en een tab per blad onderaan, kommagetallen met hoogstens vier decimalen. Json als boom, ⌥-klik klapt alles eronder. Pdf als pagina's op `bg`, getekend door CoreGraphics. Een afbeelding past in het venster, nooit groter dan hij is | 2026-11-15 |
| Documenten, bewerken | Alleen markdown en json, als platte tekst, zonder kleuren. Rechts in de titelbalk wisselt `bewerken` en `lezen`; lezen toont ook wat nog niet bewaard is. ⌘S bewaart, json alleen als het geldig is. Een tab met wijzigingen die niet bewaard zijn heeft een stip. Schrijft een sessie in het bestand en heb ik niets gewijzigd, dan neemt de editor dat over; heb ik wel iets gewijzigd, dan vraagt bewaren of ik wil overschrijven. Sluiten (tab, venster of Jera) met wijzigingen vraagt of ik wil bewaren. Andere soorten bewerk ik niet in Jera | 2026-11-15 |
| Paden in de terminal | Een pad in de terminal is aan te klikken en opent het bestand, in Documenten als het dat toont | 2026-11-15 |
| Poorten | Een lijst van de poorten waarop iets luistert, van het project in de actieve tab. Per poort: stoppen en openen | 2026-11-15 |
| Poorten, welke | Alleen wat bij het project in de actieve tab hoort: een poort, browser of artifact van een sessie die in die map liep (ook als ze niet meer draait: dan heet het wees), of een poort van een proces dat in die map is gestart. Een poort van Docker hoort bij de map waaruit docker compose de container startte. De eigen poorten van Claude Code, macOS en apps uit Applications staan er niet bij. Er is geen knop Alles. De getallen op de knop in de tabbalk tellen hetzelfde | 2026-11-15 |
| Poorten, indeling | Het paneel heeft drie kopjes: Poorten, Browsers, Artifacts. Een kopje zonder regels staat er niet. Elke regel noemt de sessie die het startte; een poort zonder sessie noemt zijn map. Een agent-browser heet naar zijn sessie, niet naar de naam die agent-browser hem geeft (zoals `default`). Vervangt de groepen per sessie | 2026-11-15 |
| Poorten, commando | Per poort staat het commando dat luistert, zoals `next dev` of `uvicorn app:main`, niet alleen de procesnaam | 2026-11-15 |
| Sessie met server of browser | In de sessieregel staat een klein icoon achter de titel zolang de sessie een poort open heeft of een agent-browser draait | 2026-11-15 |
| Stoppen met de sessie | Stopt een sessie, of sluit ik Jera, dan stopt Jera niets vanzelf. Jera weet alleen welke sessie iets startte, niet wie het nu gebruikt; een gedeelde server zou anders uitgaan terwijl een andere sessie hem nog gebruikt | 2026-11-15 |
| Wezen | Een server of agent-browser waarvan de sessie niet meer bestaat, noemt als sessie "wees". In het kopje Poorten stopt Stop wezen de wees-poorten, in Browsers sluit Sluit wezen de wees-browsers | 2026-11-15 |
| agent-browser, plaatje | In het paneel staat bij elke agent-browser een klein plaatje van wat die op dat moment toont | 2026-11-15 |
| Rechts van de terminal | Het git-paneel en het paneel met poorten en agent-browser delen één kolom van 360 px: git boven, poorten onder, helft om helft, met een lijn van 1 px ertussen. Staat er één open, dan neemt die de hele hoogte. Gekozen uit vier, https://claude.ai/artifact/Uf5cfrCSJqn3dqCbS8yRqn | 2026-11-15 |
| Artifacts | De artifacts die een sessie publiceerde staan in het paneel met poorten en agent-browser, onder het kopje Artifacts: per regel de titel, de sessie en de knop Openen, die het artifact in Safari opent. Alleen gepubliceerd telt mee, niet wat een sessie las; een nieuwe versie telt niet opnieuw. Jera leest titel en link uit het gesprek van de sessie (`toolUseResult` van publish en open). Gekozen uit vier, https://claude.ai/artifact/Xm4bVWaqrXWznqnNye2KNM | 2026-11-15 |
| Startcommando | Jera start elke sessie via mijn eigen shell met `claude`. Het is één vast commando, op één plek aan te passen | 2026-11-14 |
| Paneel voor de hele computer | Poorten en agent-browser staan samen in één paneel, los van de tabs. Een knop rechts in de tabbalk opent het en toont de aantallen | 2026-11-14 |
| Render | Per service van het project: de naam, de link naar de live app, de deploystatus en de commit die live staat. Een klik op een mislukte deploy opent de logs bij Render in Safari. Jera koppelt een service aan een tab via de repo | 2026-11-14 |
| Render, plek | Een smalle regel boven de terminal | 2026-11-14 |
| Eerste versie | Tabs, de sessielijst met status, één terminal, Bestanden, Render en het git-paneel | 2026-11-14 |
| Daarna | In deze volgorde: de oude sessies, het paneel met poorten en agent-browser, de tweede terminal ernaast, Overnemen | 2026-11-14 |
| Git | Een smal git-paneel per repo. De logica staat vast en staat in `app/src/git.rs` en `gitpaneel.rs` | 2026-11-14 |
| Git, plek | Rechts naast de terminal, als paneel dat open en dicht klapt. Het wisselt mee met de tab | 2026-11-14 |
| Thema | Een donker en een licht thema | 2026-11-14 |
| Uiterlijk, grond | Bijna zwart, één vlak, met lijnen van 1 px tussen de panelen. De terminal heeft dezelfde grond. De waarden staan in `stijl/design-system/tokens.json` | 2026-11-14 |
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
| Iconen, keuze | In de menubalk, onder Jera › Iconen, kies ik tussen Kleur (GitLab en Catppuccin) en Zwartwit (de iconen van Zed, in één kleur). Zwartwit is de standaard. Jera onthoudt de keuze | 2026-11-14 |
| Tab in de terminal | Tab en Shift-Tab gaan naar de sessie, niet naar het volgende onderdeel van het venster | 2026-11-15 |
| Git, branchnamen | De branchnamen staan boven de lijnen, getrapt: elke naam op een eigen regel | 2026-11-14 |
| Git, stijl | Kaal: een bolletje per commit (een groep is een bolletje met het aantal), het eindpunt van een branch is een vierkantje, gevuld als het de map van de tab is. Eén enkele lijn per branch; een branch splitst af met een rechte hoek. Geen datumkolom: de dag en het moment van ophalen staan rechts op hun lijn. Gekozen in https://claude.ai/artifact/ETPZ5yAZ3gJq27dmDvWods | 2026-11-15 |
| Git, merge | Een merge gaat haaks van de branch af en draait met een bocht de lijn in waar hij in ging, precies op de merge. Een gemergde branch loopt van zijn aftakpunt tot zijn merge; een squash-merge telt ook. Geen PR-nummers | 2026-11-15 |
| Git, stand op GitHub | Een wolkje op de lijn waar GitHub staat; het stuk dat nog niet gepusht is, is gestreept; wat alleen op GitHub staat is vaag gestippeld met vage bolletjes. Loopt een branch gelijk met GitHub, dan staat er niets bij. Geen lijn over alle branches: pushen gaat per branch | 2026-11-15 |
| Git, acties per branch | Een klik op de naam of het vierkantje van een branch opent zijn menu, met alleen wat daar kan: `commit n` (opent de commit view), `push ↑n` (pas als er niets te halen is), `update ↓n`, `checkout` en, bij een gemergde branch, `verwijder` (hier en op GitHub). Het menu van de hoofdbranch heeft ook `opruimen`: alle gemergde branches in één keer weg. Voorstel in https://claude.ai/artifact/ETPZ5yAZ3gJq27dmDvWods | 2026-11-15 |
| Git, wijzigingen | Wat nog niet gecommit is staat als open bolletje boven het eindpunt van zijn branch, met het aantal bestanden; nieuwe bestanden tellen mee. Een klik erop opent de commit view | 2026-11-15 |
| Git, commit view | Commit opent de commit view op de plek van de graaf, in dezelfde kolom. Bovenaan de branch, het aantal bestanden en een kruis. Daaronder de bestanden die nog niet gecommit zijn, elk met een vinkje (aan bij het openen), de soort (M, A, D of R), de naam en de map. Onderaan het bericht, dat Claude voorstelt voor de bestanden met een vinkje, en `commit` en `commit en push`. Alleen wat een vinkje heeft gaat mee; de rest blijft als wijziging staan. Enter commit, Shift+Enter is een nieuwe regel, Escape of het kruis gaat terug naar de graaf. `commit en push` staat er alleen als de repo op GitHub staat en er niets te halen is. Gekozen uit het git-paneel, een groter menu bij de klik en een eigen venster | 2026-11-15 |
| Git, checkout | Wat in de map niet gecommit is, blijft geparkeerd bij zijn branch (`git stash`, bericht `jera`) en komt terug bij een checkout terug. Geen checkout zolang Claude in de repo bezig is. Gekozen uit parkeren, meenemen, eerst committen en een nieuwe tab | 2026-11-15 |
| Git, inklappen | De knop git in de Render-regel klapt het paneel in en uit. Bij het openen van Jera is het ingeklapt | 2026-11-14 |
| Git, ophalen | Het paneel haalt zelf op bij GitHub, bij openen en elke 5 minuten zolang het open is | 2026-11-15 |
| Documenten | Geen viewers in de app. Een klik op een document opent het direct in een externe app, zoals Safari, Preview of GridCraft | 2026-11-14 |
| Openen | `+ map` en ⌘O openen het scherm Openen. Het hangt direct onder de tab `+ map`, tegen de titelbalk aan. Bovenaan `open` (met een mapicoon, opent de mapkeuze van macOS) en `clone repository` (met het icoon van git), daaronder het zoekveld en de mappen die ik het laatst in Jera opende. Typen filtert op de naam en kiest de eerste die past, pijltjes en Enter kiezen. Zonder tabs staat Openen open | 2026-11-15 |
| Openen, per regel | Eén kolom: de naam, en klein eronder het pad. Staat het project al als tab open, dan springt kiezen naar die tab | 2026-11-15 |
| Openen, klonen | `clone repository` toont onder het zoekveld mijn repo's op GitHub in plaats van de recente mappen. Jera kloont de gekozen repo in een map die ik elke keer kies, en start er daarna een sessie. Escape of nog een klik op `clone repository` gaat terug | 2026-11-15 |
| Tab zonder sessies | Stopt de laatste sessie van een tab, dan blijft de tab open. Op de plek van de terminal staat "Geen sessies." met de knop Sessie starten, die een sessie start in de hoofdmap. Bestanden en het blok oud werken gewoon. Alleen het kruisje sluit een tab | 2026-11-15 |
| Nieuw project | Alleen met ⌘N en in de menubalk: ik typ een naam en kies daarna de map waarin het komt. Jera maakt de map, doet `git init` en start een sessie. Alleen lokaal | 2026-11-15 |
| Op GitHub | Heeft een repo geen remote, dan toont het git-paneel de eigenaar (een klik wisselt) en de knop Op GitHub. Die maakt een privé repo en zet de code erop | 2026-11-15 |
| Naam | Jera, naar de rune ᛃ voor oogst: een knipoog naar Hidden Harvest. Het teken is het icoon van de app | 2026-11-15 |
| Taal | Nederlands en Engels. In de menubalk, onder Jera › Taal, kies ik tussen Nederlands en English. Zonder keuze volgt Jera de eerste taal van macOS: Nederlands als die Nederlands is, anders Engels | 2026-11-15 |
| Icoon | Het teken ᛃ met rechte uiteinden. Donker: wit `#F4F5F0` op het groen van Hidden Harvest `#1D3A2A`. Licht: omgekeerd. Staat in `stijl/icoon/`. In het Dock wisselt het mee met macOS; links in de titelbalk, in de tab van de Jera-sessie, staat het teken in `ink` | 2026-11-15 |
| Git, teken op de merge | Waar een branch een lijn in gaat staat een ruit in de kleur van de branch die binnenkomt, in plaats van het bolletje: 8 px van punt tot punt. Is de merge het eindpunt van de lijn, dan staat dezelfde ruit in het vierkant. De bocht en de lijnen blijven. Gekozen uit drie, https://claude.ai/artifact/WkjjYE5eSDJeTbDx8nPbih | 2026-11-20 |
| Uiterlijk, linkerkolom | De linkerkolom heeft het vlak `bar`, zoals de titelbalk; de lijn van 1 px ernaast blijft. Zo staat ze los van de terminal. Voor de kolom vervangt dit "één vlak" uit Uiterlijk, grond. Gekozen uit drie, https://claude.ai/artifact/WkjjYE5eSDJeTbDx8nPbih | 2026-11-20 |
| Jera-sessie | Een Claude-sessie in mijn thuismap, voor de hele computer, los van de projecten. Links in de titelbalk, voor de projecttabs, staat haar vaste tab: het teken ᛃ, de naam `kernel` en de status van de sessie. De tab loopt tot de lijn van de linkerkolom, zodat de projecttabs boven de terminal beginnen. Een klik start de sessie, of gaat naar haar tab als ze al draait. De tab heeft geen aantal en geen kruisje, en de thuismap staat niet bij de recente mappen in Openen. Het blok oud toont daar alleen de sessies uit de thuismap zelf | 2026-11-20 |
| Git, zijtak | Een branch is een zijtakje van zijn basis: van de commit waar hij de basis verliet tot zijn eindpunt, met alleen zijn eigen commits. Dat blijft zo als hij de basis tussendoor in zich opnam en als er een andere branch aan hangt. Een vaste branch en een branch zonder basis lopen door tot onderaan. Is het eindpunt de merge van de basis, dan komt de bocht daar aan, met de ruit in het vierkant. De tests staan in `app/src/gitpaneel.rs` | 2026-11-20 |

Het design system staat in `stijl/design-system/` en op https://claude.ai/artifact/QKTtDToW7LYRivNM1vYG88: de tokens en een voorbeeld per onderdeel. De feiten bij de stack en de terminal staan in `research/native.md`. De sessiestatus leest Jera uit het sessiebestand van Claude Code; de meting staat in `research/sessies.md`.

### Indeling

Schakelaar op Sessies:

```
[ webshop  3 ]  [ jera  1 ]  [ + map ]                         [ Poorten 4  agent-browser 1 ]
─────────────────────────────────────────────────────────────────────────────────────────────
 [ Sessies ]  Bestanden           │ ● api  api.onrender.com    ● web  web.onrender.com
 webshop/                    [+]   │──────────────────────────────────────┬─────────────────
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
 [ Sessies ]  Bestanden                  │            │ kpi cards
 webshop/                           [+]  │            │   3000  next dev        [Openen] [Stoppen]
   ● kpi cards                     ◫ ⌁  │            │   agent-browser default         [Sluiten]
                                         │  terminal  │   ┌──────────────┐
─────────────────────────────────────────│            │   │   plaatje    │
 ▾ oud  3                                │            │   └──────────────┘
 gisteren                            ▾  │            │ zonder sessie
   supabase migratie                     │            │   8000  uvicorn app     [Openen] [Stoppen]
 11 nov                              ▾  │            │ wezen                      [Stop alle]
   login flow                            │            │   5173  vite            [Openen] [Stoppen]
   tabel sorteren                        │            │
```

De schermen staan ook in het artifact https://claude.ai/artifact/7yXL3y5ykQbPFcZ4GXGXrX (het bord "Gekozen").

## Open

- **Render:** later misschien: logs en belasting in Jera zelf, en knoppen om te deployen of te herstarten.
- **Terminal:** tekent Claude Code goed (zie `research/native.md`); selecteren en kopiëren werken. Nog te bouwen: een pad aanklikken, letters met een accent, en bloktekens zonder naad.

## Later

- Meldingen.
