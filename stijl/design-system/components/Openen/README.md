Het scherm waarin ik een map open, een repo van GitHub kloon of een recente map kies.

**Status:** voorstel, 15 november 2026. Gebouwd in de app, nog niet gekozen.

Geef de mappen die ik het laatst in Jera opende, en de repo's op GitHub.

- Een vak op `bg`, met een rand van `lijn-dikte` in `line-strong`. Rechte hoeken, geen schaduw. Het hangt direct onder de tab `+ map`: de bovenrand valt samen met de lijn onder de titelbalk, de linkerrand met de lijn links van de tab. Het ligt boven de rest; klikken ernaast of Escape sluit het.
- Bovenaan twee regels van `h-rij` hoog, met een icoon van `icoon` groot: `open` met het icoon van een map, `clone repository` met het icoon van git, beide uit de set van de bestandsboom. Daarna een lijn in `line`.
- Daaronder het zoekveld, met een lijn in `line` eronder als er een lijst volgt. Typen filtert de lijst: elk woord moet in de naam staan. De eerste regel die past is gekozen. Past er niets, dan staat er één regel in `mute`: `Geen map met deze naam.`
- Daaronder de recente mappen, de laatste eerst. Eén kolom: de naam, en klein eronder (11 px, `mute`) het pad met ~ voor de thuismap. Zo'n regel is hoger dan `h-rij`, omdat er twee regels tekst in staan.
- `clone repository` toont onder het zoekveld mijn repo's op GitHub in plaats van de recente mappen, op dezelfde manier: de naam, en klein eronder eigenaar/naam. De regel `clone repository` is dan vet. Tijdens het klonen staat onder de naam `klonen`, bij een fout `bestaat al` of `mislukt`. Escape of nog een klik op `clone repository` gaat terug.
- De gekozen regel heeft het vlak `sel` met de streep in `accent` links, zoals een sessieregel. Bij het openen is dat de eerste van de lijst. Pijltjes verplaatsen hem, ook naar de twee regels bovenaan. Enter of een klik kiest.
- Nieuw project (⌘N) toont alleen het veld, voor de naam, met rechts wat er mis is: `geen naam`, `geen / in een naam`, `bestaat al`.
- Kan Jera GitHub niet lezen, dan staat er één regel in `mute`: `GitHub niet bereikbaar. Log in met gh auth login.`
