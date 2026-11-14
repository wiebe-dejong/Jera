Cockpit is een Mac-app waarin één persoon zijn Claude Code-sessies draait en overziet. Het venster moet in één blik zeggen welke sessie op hem wacht. Alles hieronder dient dat.

## Taal

- Schrijf Nederlands, in kleine letters waar het een label is: `poorten`, `git`, `agent-browser`.
- Gebruik voor de status altijd deze vijf woorden: bezig, wacht op antwoord, wacht op toestemming, net klaar, idle. Een agent heeft er drie: bezig, klaar, gestopt.
- Zet geen zin in het scherm die het scherm uitlegt. Wat ertoe doet is een label, een kolom of een icoon.
- Toegestaan als één korte regel: een leeg scherm ("Nog geen sessie in deze map."), een fout met wat te doen, en de vraag in een dialoog.

## Kleur

- Het venster is één vlak in `bg`. Scheid panelen met een lijn van `lijn-dikte` in `line`, nooit met een ander vlak of een schaduw.
- Alleen de titelbalk (`bar`) en de gekozen rij (`sel`) hebben een eigen vlak.
- Tekst is `ink`. Wat minder telt is `mute`: mapnamen, tellers, niet-actieve tabs, idle sessies.
- `accent` is alleen een streep van `streep` dik. Die zegt wat actief of gekozen is. Maak er nooit een vlak of tekst van.
- `ok`, `warn` en `bad` zijn alleen status. Gebruik ze nergens als versiering.
- `warn` is alleen voor iconen. Zet er geen tekst in.
- De kleuren `lane-1` tot `lane-4` horen bij de lijnen van het git-paneel en komen nergens anders voor.
- De kleuren `team-blue` tot `team-red` zijn alleen voor de naam van een teamlid in Agents, op `bg`. Claude Code geeft elk teamlid een van deze acht kleuren. Groen, geel en rood lijken op de status; het icoon ervoor zegt de stand, de naam niet.
- Elk onderdeel werkt in donker en licht met dezelfde namen. Gebruik nooit een losse kleurwaarde.

## Letter

- Eén letter voor alles: JetBrains Mono, via `mono`.
- Zet tekst in `tekst`. Maak vet (`tekst-vet`) wat actief of gekozen is, en de naam van een service.
- De terminal gebruikt `terminal`, met meer regelhoogte.
- Geen hoofdletters als stijl, geen cursief, geen andere maten dan in de tokens.

## Maat en vorm

- Elke rij in een lijst is `h-rij` hoog en begint op `pad` van de rand.
- Tussen een icoon en zijn tekst zit `gap`.
- Alles is recht (`hoek`). Geen ronde hoeken, geen schaduw.
- Een gekozen rij loopt van rand tot rand van zijn kolom.

## Status

Elke sessie toont één icoon van `icoon` groot, voor de titel, zonder woord. Het woord is de tooltip en het label voor schermlezers.

| Status | Icoon | Kleur |
|---|---|---|
| bezig | draaiend boogje | `ok` |
| wacht op antwoord | rondje met vraagteken | `warn` |
| wacht op toestemming | slot | `bad` |
| net klaar | vinkje | `ok` |
| idle | streepje | `mute` |

- Elke status heeft een eigen vorm. Vertrouw nooit op kleur alleen.
- Een idle sessie heeft ook een gedempte titel.
- Een service bij Render gebruikt dezelfde taal: een vinkje in `ok` voor live, een kruis in `bad` voor een mislukte deploy.
- Een agent ook: het boogje in `ok` voor bezig, een vinkje in `ok` voor klaar, een streepje in `mute` voor gestopt. Een teamlid houdt zijn kleur, ook als hij klaar of gestopt is.

## Iconen

- Iconen zijn lijntekeningen van 14 px op een raster van 16, lijn 1.8 tot 2.2 dik, ronde uiteinden.
- Uitzondering: de iconen van bestanden en mappen in de bestandsboom komen uit Zed, in `mute`, of in kleur uit GitLab en Catppuccin.
- Er zijn alleen statusiconen, het kruis, het pijltje van een map, het plusteken en de iconen van bestanden en mappen in de bestandsboom. Voeg geen icoon toe als een woord volstaat.
- Geen emoji.

## Beweging

- Alleen het boogje van bezig draait, 1,1 s per rondje.
- Bij "verminder beweging" in macOS staat het stil.

## Onderdelen

Alle onderdelen zijn gekozen: Tab, Schakelaar, Sessieregel, Agents (de teamleden en het aantal subagents onder een sessie), Statusicoon, RenderRegel, Terminal, GitPaneel, Knop, Bestandsboom, EerderRegel (de regel in het blok oud), Poortenpaneel, Dialoog en LegeStaat.

- Voorstel: Contextmenu, het menu van een twee-vingerklik in de bestandsboom. Al gebouwd in de app.
- Voorstel: Openen, het scherm waarin ik een project kies of een nieuw maak. Al gebouwd in de app.
- Verander niets aan een onderdeel zonder nieuwe keuze.
- Bouw een nieuw scherm uit bestaande onderdelen. Is er een nieuw onderdeel nodig, voeg het dan eerst hier toe als voorstel.
