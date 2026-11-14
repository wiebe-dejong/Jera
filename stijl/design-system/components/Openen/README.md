Het scherm waarin ik een map open, een repo van GitHub kloon of een recente map kies.

**Status:** voorstel, 15 november 2026. Gebouwd in de app, nog niet gekozen.

Geef de mappen die ik het laatst in Cockpit opende, en de repo's op GitHub.

- Een vak midden in het venster, op `bg`, met een rand van `lijn-dikte` in `line-strong`. Rechte hoeken, geen schaduw. Het ligt boven de rest; klikken ernaast of Escape sluit het.
- Bovenaan het zoekveld, met een lijn in `line` eronder. Typen filtert de lijst: elk woord moet in de naam of het pad staan.
- Daaronder twee regels van `h-rij` hoog, met een icoon van `icoon` groot: `open` met het icoon van een map, `clone repository` met het icoon van git, beide uit de set van de bestandsboom. Daarna een lijn in `line`.
- Daaronder de recente mappen, de laatste eerst. Eén kolom: de naam, en klein eronder (11 px, `mute`) het pad met ~ voor de thuismap. Zo'n regel is hoger dan `h-rij`, omdat er twee regels tekst in staan.
- `clone repository` toont in hetzelfde vak mijn repo's op GitHub, op dezelfde manier: de naam, en klein eronder eigenaar/naam. Tijdens het klonen staat daar `klonen`, bij een fout `bestaat al` of `mislukt`. Escape gaat terug.
- De gekozen regel heeft het vlak `sel` met de streep in `accent` links, zoals een sessieregel. Pijltjes verplaatsen hem, Enter of een klik kiest.
- Nieuw project (⌘N) toont alleen het veld, voor de naam, met rechts wat er mis is: `geen naam`, `geen / in een naam`, `bestaat al`.
- Kan Cockpit GitHub niet lezen, dan staat er één regel in `mute`: `GitHub niet bereikbaar. Log in met gh auth login.`
