Jera is een Mac-app waarin één persoon zijn Claude Code-sessies draait en overziet. Het venster moet in één blik zeggen welke sessie op hem wacht. Alles hieronder dient dat.

## Taal

- Schrijf Nederlands, in kleine letters waar het een label is: `poorten`, `git`, `agent-browser`.
- Gebruik voor de status altijd deze vijf woorden: bezig, wacht op antwoord, wacht op toestemming, net klaar, idle. Een agent heeft er drie: bezig, klaar, gestopt.
- Zet geen zin in het scherm die het scherm uitlegt. Wat ertoe doet is een label, een kolom of een icoon.
- Toegestaan als één korte regel: een leeg scherm ("Nog geen sessie in deze map."), een fout met wat te doen, en de vraag in een dialoog.

## Kleur

- Het venster is één vlak in `bg`. Scheid panelen met een lijn van `lijn-dikte` in `line`, nooit met een ander vlak of een schaduw.
- Alleen de titelbalk (`bar`), de gekozen rij en wat onder de muis staat (`sel`) hebben een eigen vlak.
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
- De cursor knippert waar ik typ, in de terminal en in een invoerveld: een halve seconde aan, een halve seconde uit. Tijdens het typen en zonder focus knippert hij niet.

## Onder de muis

- Een rij die aan te klikken is krijgt het vlak `sel`, zonder streep en zonder vet: een sessieregel, een agent, de kop `oud`, een dag en een sessie in het blok oud, een regel in de bestandsboom, in Openen en in het Contextmenu.
- Een woord of teken in `mute` dat aan te klikken is wordt `ink`: een niet-actieve tab, `+ map`, `sessies` en `bestanden`, `git`, het plusteken bij een map, het pijltje van agents.
- Een knop met een rand krijgt het vlak `sel`. De eigenaar in het git-paneel ook, zo hoog als de knop ernaast.
- Het statusicoon van een sessie toont na een halve seconde zijn woord in een kaartje: `bg` met een rand in `line-strong`. Een agent toont zo zijn opdracht en wat hij doet, zie Agents.
- De gekozen rij en het actieve woord veranderen niet. Wat niet aan te klikken is ook niet.
- De muisaanwijzer blijft een pijl.

## Onderdelen

Alle onderdelen zijn gekozen: Tab, Schakelaar, Sessieregel, Agents (de agents onder een sessie: de subagents die bezig zijn en de teamleden), Statusicoon, RenderRegel, Terminal, GitPaneel, Knop, Bestandsboom, EerderRegel (de regel in het blok oud), Poortenpaneel, Dialoog en LegeStaat.

- Voorstel: Contextmenu, het menu van een twee-vingerklik in de bestandsboom. Al gebouwd in de app.
- Voorstel: Openen, het scherm waarin ik een project kies of een nieuw maak. Al gebouwd in de app.
- Voorstel: Statusbalk, de rij onderaan rechts van de linkerkolom: `h-rij` hoog, op `bg`, met een lijn erboven, op één lijn met de kop `oud`. Rechts de teller: `poorten`, `browsers` en `artifacts` met hun aantal; een soort met 0 staat er niet. Een klik opent het Poortenpaneel als lade boven de teller, over de terminal, rechts tegen de rand. Gekozen op het bord in `stijl/server/`. Al gebouwd in de app.
- Voorstel: Documenten, het venster dat markdown, tabellen, json, afbeeldingen en pdf toont. Al gebouwd in de app.
- Verander niets aan een onderdeel zonder nieuwe keuze.
- Bouw een nieuw scherm uit bestaande onderdelen. Is er een nieuw onderdeel nodig, voeg het dan eerst hier toe als voorstel.
