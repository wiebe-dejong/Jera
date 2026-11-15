Het overzicht van de branches van de repo, rechts naast de terminal.

**Status:** gekozen op het keuzebord. De actieregel, de muis en het kaartje: voorstel, gekozen op het keuzebord Git-paneel acties (`stijl/git-acties/`).

Geef de branches met hun commits, de dagen, en het moment waarop voor het laatst is opgehaald.

- De branchnamen staan boven de lijnen, getrapt: elke naam op een eigen regel in `branchnaam`, de hoofdbranch bovenaan, in de kleur van zijn lijn.
- Van elke naam loopt een dunne lijn naar het punt van de branch.
- Er is geen datumkolom. Elke dag begint met een lijn in `line`; de naam van de dag staat rechts eronder in `klein` en `mute`. `opgehaald` met het tijdstip staat rechts op een gestippelde lijn in `line-strong`, in `warn` als het langer dan een dag geleden is.
- Alles is recht, net als de rest van Jera: een commit is een vierkantje van 6 px, een groep van drie of meer commits een vierkantje van 10 px met het aantal ernaast, het eindpunt van een branch een hol vierkant van 13 px. Een merge is een hol vierkantje. Een branch splitst af met een rechte hoek. De lijnen zijn 1.5 px.
- Een lijn is vol als het stuk lokaal en op GitHub staat, gestippeld als het op één van de twee nog ontbreekt (nog te pushen of nog te pullen), en hol als de hele branch nog nooit gepusht is. Waar de branch op GitHub staat, staat een wolkje. Gekozen op het keuzebord GitHub-stand (`stijl/github-stand/`).
- De lijnen gebruiken `lane-1` tot `lane-4`, in die volgorde.
- De breedte volgt uit het aantal lijnen en de langste naam.
- De knop `git` in de Render-regel klapt het paneel in. Dan krijgt de terminal de volle breedte.
- De graaf zelf (bolletjes, pijlen, groepen met een getal) ligt vast in `app/src/gitpaneel.rs` van het project.
- Boven de graaf staat de actieregel: alleen wat er te doen is, als knoppen (`commit 3`, `update ↓2`, `push ↑1`), op `pad` van de rand, 34 px hoog, met een lijn in `line` eronder. Is er niets te doen, dan is er geen regel. Push staat er pas als er niets te halen is.
- `commit` opent in die regel een invoerveld met de knop `commit`. Claude schrijft een voorstel (`…` zolang het schrijft); wat ik intussen typ blijft staan. Enter commit alles, Escape sluit het veld. `update` haalt binnen met rebase; bij een conflict blijft alles zoals het was en zegt de regel wat te doen. Tijdens het werk staat er `committen`, `pushen` of `updaten` in `mute`.
- Het paneel haalt zelf op: bij het openen en elke 5 minuten zolang het open is.
- Onder de muis: het dichtstbijzijnde punt binnen 30 px opzij en 20 px boven of onder krijgt een kader van 18 px, 1 px in `ink`, en de andere lijnen gaan naar 30%. Geen tekst. Gaat de muis eraf, dan laat het na 150 ms los.
- Een klik op een punt toont het kaartje ernaast: `bg`, rand in `line-strong`, links de streep in `accent`, zo breed als de tekst en hoogstens 290 px. Bij een commit: wie en hoe laat (`Wiebe de Jong · 12:47`, met de datum als het niet vandaag is). Bij een branch of het wolkje: de naam, en hoeveel commits hij achterloopt op de hoofdbranch. Escape, een klik op een leeg vlak of nog een klik op het punt haalt het weg.
- Heeft de repo branches van agents (`worktree-agent-…`), dan staat rechtsboven in de kop een knop met hun aantal; die klapt ze in en uit.
