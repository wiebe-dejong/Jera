Wat er in het git-paneel staat zolang ik commit: de bestanden die nog niet gecommit zijn, het bericht en de knoppen.

**Status:** voorstel, 15 november 2026. Gebouwd in de app, nog niet gekozen.

Geef de branch, de bestanden die nog niet gecommit zijn met hun soort, en of de repo op GitHub staat.

- De view staat op de plek van de graaf en is even breed. Hij opent met een klik op het open bolletje van een branch of op `commit` in zijn menu.
- De kop is `h-rij` plus 8 px hoog, met een lijn in `line` eronder: de branch in vet, rechts het aantal bestanden in `mute` (`5 bestanden`, of `4 van 5 bestanden` als er een vinkje uit staat) en het kruis in `mute`, dat onder de muis `ink` wordt.
- Elk bestand is een regel van `h-rij`: het vinkje, de soort in `mute` (`M` gewijzigd, `A` nieuw, `D` verwijderd, `R` hernoemd), de naam in `ink` en de map in `mute`. Wat niet past, eindigt op `…`; de map wijkt eerst.
- Het vinkje is een vakje van `icoon` groot met een rand in `line-strong`, met het vinkje van net klaar erin, in `ink`. Bij het openen staan alle vinkjes aan. Een klik op de regel zet het aan of uit; de regel krijgt onder de muis het vlak `sel`. Zonder vinkje staat de naam in `mute`.
- Onderaan, boven een lijn in `line`: het veld voor het bericht, met een rand in `line-strong`, en rechts eronder de knoppen `commit` en `commit en push`, beide gewoon.
- Het veld is één regel hoog en groeit mee met het bericht, tot vijf regels. Zolang Claude het voorstel schrijft staat er `…`. Het voorstel gaat over de bestanden met een vinkje en volgt de vinkjes, tot ik zelf typ.
- Enter commit, Shift+Enter is een nieuwe regel, Escape of het kruis gaat terug naar de graaf.
- `commit en push` staat er alleen als de repo op GitHub staat en er niets te halen is.
- Kan een knop niets doen (geen vinkje aan, of Jera is bezig), dan staat het woord in `mute` en verandert hij niet onder de muis.
- Wat Jera doet (`committen main`, `pushen main`) en wat er misging staat in de regel bovenaan het paneel, zoals bij de andere acties.
