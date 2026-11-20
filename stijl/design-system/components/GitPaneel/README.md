Het overzicht van de branches van de repo, rechts naast de terminal.

**Status:** gekozen op het keuzebord.

Geef de branches met hun commits, de dagen, en het moment waarop voor het laatst is opgehaald.

- De branchnamen staan boven de lijnen, getrapt: elke naam op een eigen regel in `branchnaam`, de hoofdbranch bovenaan, in de kleur van zijn lijn.
- Van elke naam loopt een dunne lijn naar het punt van de branch.
- Er is geen datumkolom. Elke dag begint met een lijn in `line`; de naam van de dag staat rechts eronder in `klein` en `mute`. `opgehaald` met het tijdstip staat rechts op een gestippelde lijn in `line-strong`, in `warn` als het langer dan een dag geleden is.
- Alles is recht, net als de rest van Jera: een commit is een vierkantje van 6 px, een groep van drie of meer commits een vierkantje van 10 px met het aantal ernaast, het eindpunt van een branch een hol vierkant van 13 px. Een merge is een ruit in de kleur van de branch die binnenkomt: 13 px op de lijn, 8 px in het vierkant als de merge het eindpunt is. Een branch splitst af met een rechte hoek. De lijnen zijn 1.5 px.
- Een lijn is vol als het stuk lokaal en op GitHub staat, gestippeld als het op één van de twee nog ontbreekt (nog te pushen of nog te pullen), en hol als de hele branch nog nooit gepusht is. Waar de branch op GitHub staat, staat een wolkje. Gekozen op het keuzebord GitHub-stand (`stijl/github-stand/`).
- De lijnen gebruiken `lane-1` tot `lane-4`, in die volgorde.
- De breedte volgt uit het aantal lijnen en de langste naam.
- De knop `git` in de Render-regel klapt het paneel in. Dan krijgt de terminal de volle breedte.
- De graaf zelf (bolletjes, pijlen, groepen met een getal) ligt vast in `app/src/gitpaneel.rs` van het project.
- Het paneel haalt zelf op: bij het openen en elke 5 minuten zolang het open is.
- Heeft de repo branches van agents (`worktree-agent-…`), dan staat rechtsboven in de kop een knop met hun aantal; die klapt ze in en uit.
