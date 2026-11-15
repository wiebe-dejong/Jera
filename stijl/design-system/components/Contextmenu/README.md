Het menu van een twee-vingerklik op een regel in de bestandsboom, en het veld waarin ik daarna een naam typ.

**Status:** voorstel, 15 november 2026. Gebouwd in de app, nog niet gekozen.

Geef het pad waarop geklikt is en wat er op het klembord van macOS staat.

- Het menu staat op de plek van de klik, op `bg`, met een rand van `lijn-dikte` in `line-strong`. Rechte hoeken, geen schaduw.
- Elke regel is `h-rij` hoog en begint op `pad`, met een icoon van `icoon` groot in `mute`, dan `gap`, dan het woord. Onder de muis krijgt een regel het vlak `sel`.
- De regels, in drie groepen gescheiden door een lijn in `line`: `nieuw` · `kopiëren`, `plakken` · `naam wijzigen`, `naar prullenmand`.
- Bij een bestand dat Documenten toont, staat daarboven `in app openen`, met het icoon van het bestand uit de bestandsboom, en een lijn. Het opent het bestand in de app die macOS ervoor heeft.
- Achter `nieuw` staat het pijltje van een dichte map. Onder de muis gaat rechts ernaast een tweede vak open met de soorten: `map`, een lijn, dan `bestand` en de soorten van bestanden met hun icoon uit de bestandsboom en rechts de extensie in `mute`. Zolang dat vak open is, heeft `nieuw` het vlak `sel`.
- De iconen van de acties zijn eigen lijntekeningen: plus, twee vellen, klembord, potlood, prullenbak. Dit is een uitzondering op "voeg geen icoon toe als een woord volstaat", op verzoek.
- Zolang het menu open is, is de regel waar het bij hoort gekozen: vlak `sel`, `accent`-streep, vet. Elke regel in de bestandsboom krijgt onder de muis het vlak `sel`.
- `plakken` staat in `mute` en doet niets als er geen bestand op het klembord staat.
- Bij de hoofdmap ontbreken `naam wijzigen` en `naar prullenmand`.
- Nieuw en plakken gaan in de map zelf, of in de map waar het bestand in staat.
- Kopiëren en plakken gaan via het klembord van macOS, dus ook van en naar Finder. Een naam die er al is krijgt `kopie` erachter, zoals in Finder. Plakken loopt op de achtergrond. Zolang het loopt, draait het boogje van bezig (`ok`) rechts in de regel van de map, voor het plusteken; daarna ververst de map.
- Het naamveld staat in de regel zelf, met een rand in `line-strong`. Bij naam wijzigen is de naam zonder extensie gekozen; bij een nieuw bestand staat de extensie er al en staat de cursor ervoor. Enter of ergens anders klikken bevestigt, Escape annuleert. Wat er mis is staat rechts in `mute`: `bestaat al`, `geen naam`, `geen / in een naam`.

**Bij een sessie** (voorstel, 15 november 2026): een twee-vingerklik op een sessieregel.

- Bovenaan wat er over de sessie te weten is, een regel per feit: het woord in `mute` links, de waarde rechts in `ink`. Nu alleen `looptijd` (`40 s`, `12 min`, `1 u 12 min`).
- Een lijn in `line`, dan `naam wijzigen` (potlood) en `stoppen` (vierkantje), `stoppen` in `bad`.
- Naam wijzigen opent het naamveld in de sessieregel, met de titel gekozen. Stoppen stopt Claude meteen; de sessie staat daarna bij oud.
