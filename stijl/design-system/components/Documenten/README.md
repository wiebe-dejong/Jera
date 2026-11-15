Het venster dat markdown, tabellen (csv, xlsx), json, afbeeldingen en pdf toont, met een tab per bestand.

**Status:** voorstel, 15 november 2026. Gebouwd in de app, nog niet gekozen.

Geef het pad van het bestand. Een klik in de bestandsboom en een ⌘-klik op een pad in de terminal openen het hier; wat Documenten niet toont, gaat naar de app die macOS ervoor heeft.

- Een eigen venster, één vlak in `bg`, met de titelbalk van het hoofdvenster: `bar`, `h-titelbalk` hoog, ruimte voor de drie knoppen van macOS.
- Per bestand een Tab: het icoon van het bestand uit de bestandsboom, `gap`, de naam. Onder de muis staat rechts het kruis. Hebben twee tabs dezelfde naam, dan staat de map erachter in `mute`. Te weinig ruimte: de naam wordt afgekapt met …
- Rechts in de titelbalk `in app openen`, in `mute`, onder de muis `ink`. Bij markdown en json staat daar `bewerken`, en tijdens het bewerken `lezen`.
- Heeft een tab wijzigingen die niet bewaard zijn, dan staat na de naam een stip in `mute`; onder de muis wordt die het kruis.
- Verandert het bestand op schijf, dan toont de tab de nieuwe versie; de plek blijft.
- **Markdown**: hoogstens 760 px breed, in het midden. Koppen vet in `tekst`, even groot als de tekst. Links in `ink`, onderstreept. Inline code op `bar`. Een codeblok en een tabel met een rand in `line`, de kop van een tabel op `bar` en vet. De taal van een codeblok rechtsboven in `mute`.
- **Tabel** (csv, tsv, xlsx, xls, ods): rijen van `h-rij` met een lijn in `line` eronder en tussen de kolommen. De kopregel vet, met een lijn in `line-strong`. Links de rijnummers in `mute`. Een kolom is zo breed als zijn langste waarde, hoogstens 40 tekens; wat langer is, krijgt …. Een kolom met alleen getallen staat rechts. Een werkboek heeft letters boven de kolommen, zoals in Excel, en onderaan een balk op `bar` met een Tab per blad.
- **Json**: de regels van de Bestandsboom, `h-rij` hoog, `inspring` per niveau, met het pijltje van een map. De sleutel in `ink`, dan `:` in `mute`, dan de waarde zoals in json. Een object of lijst toont alleen zijn telling in `mute`: `{3}`, `[12]`. De plek in een lijst (`0`, `1`) in `mute`. Een object of lijst krijgt onder de muis het vlak `sel`; een klik klapt hem open of dicht, met ⌥ ook alles eronder.
- **Afbeelding**: past in het venster met 24 px eromheen, nooit groter dan hij is.
- **Pdf**: de pagina's onder elkaar op `bg`, hoogstens 900 px breed, 24 px ertussen.
- Een fout is één regel in `mute` in het midden: `Geen geldige json: regel 3, teken 14.`
- **Bewerken** (markdown en json): de platte tekst in JetBrains Mono, in `ink` op `bg`, zonder kleuren. Markdown staat even breed en op dezelfde plek als bij lezen en loopt door op de volgende regel. Json heeft regelnummers in `mute`. Een selectie in `line-strong`.
- Lukt bewaren niet, dan staat de fout als één regel in `mute` in de titelbalk, links van `lezen`: `Geen geldige json: regel 3, teken 14.` Bij de volgende toets gaat hij weg.
