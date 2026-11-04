# Cockpit: spec

Spec-fase. Indeling en uiterlijk zijn nog niet besproken. Niets is definitief tenzij de status **besloten** is.

## Doel

Eén Mac-app voor mijn dagelijkse ontwikkelwerk. Die vervangt:
- PyCharm, als host voor Claude-terminals;
- Safari en Chrome, voor localhost en de live apps;
- Preview en Excel, om te bekijken;
- de dashboards van Render en Supabase.

Geen IDE: voor de debugger blijft PyCharm.

## Uitgangspunt

**Claude voert uit, de cockpit laat zien en laat mij ingrijpen.**

Uit `research/werkwijze.md`:
- Claude doet het werk: start zelf de dev-servers, maakt ~77% van de commits, pusht, en regelt deploys en database via MCP.
- Ik draai vaak 7 sessies tegelijk.
- Ik controleer in Safari.
- Servers en poorten raken een rommel: ~25 poorten, honderden keren `kill` en `lsof`.

## Onderdelen

1. **Claude-sessies**: terminals per project, met status: bezig, wacht op mij of klaar.
2. **Servers en poorten**: ook servers die Claude zelf startte. Stoppen, herstarten, achtergebleven servers opruimen.
3. **Browser**: gedeeld met Claude (agent-browser via CDP), met een eigen sessie per project zodat het juiste GitHub-account ingelogd blijft.
4. **Documenten**: md, pdf, csv, xlsx en afbeeldingen.
5. **Git**: status, diff, historie.
6. **Deploys**: deploystatus op Render en health van Supabase, per project.

## Keuzes tot nu toe

| Onderwerp | Keuze | Status |
|---|---|---|
| Zijbalk | Alleen projecten. Per project: sessies, servers, deploystatus | voorlopig, onderdeel van indeling |
| Hoofdscherm | Vaste zones: zijbalk, terminals, browser of viewer, statusbalk onderaan | voorlopig, indeling nog volledig bespreken |
| Browser | Gedeeld: Claude opent tabs in de cockpit-browser, gemarkeerd met de sessienaam. Ik kijk mee en kan overnemen | voorlopig |
| Review na een sessie | Niet in de MVP | voorlopig |
| Stack | Electron, React, TypeScript, Tailwind | advies uit onderzoek |
| IDE | Geen. PyCharm blijft voor de debugger, met een actie "open in PyCharm op regel" | advies uit onderzoek |
| Ontwikkelen | In PyCharm, totdat de cockpit zelf sessies en terminals heeft | besloten |

## Nog te bespreken

1. **Indeling, volledig:**
   - de zones;
   - tabs en splits;
   - de statusbalk;
   - wat waar staat;
   - het gedrag op een laptopscherm.
2. **Flows.** Kandidaten:
   - project openen en de werkruimte herstellen;
   - nieuwe taak, waarbij de sessie start met mijn startcommando en de taaknaam;
   - een sessie wacht op mij;
   - resultaat bekijken: server, browser, document;
   - deploy volgen na een push;
   - servers opruimen.
3. **Klikbaar prototype**: een HTML-pagina met mijn echte projectnamen en poorten.
4. **Uiterlijk** via `/stijlkeuze`: kleur, dichtheid, lettertype, statusindicatoren.
5. **MVP-scope** vastzetten.

## MVP (concept uit het onderzoek, nog niet besloten)

- Projectzijbalk.
- Terminal-tabs voor Claude, met een knop "nieuwe sessie" (startcommando en taaknaam).
- Sessiestatus via Claude Code-hooks.
- Servers:
  - ontdekken via `lsof`;
  - starten, stoppen, herstarten;
  - logs en poorten zien.
- Browser met DevTools en CDP.
- Viewer voor md, pdf, csv en afbeeldingen. Xlsx via Quick Look.
- Config die direct herlaadt, een command palette en een layout per project.

## Niet bouwen

- De kern van de terminal.
- Een eigen git-implementatie.
- LSP of een debugger.
- Een spreadsheet- of PDF-editor.
- Een databaseclient.
- Een plugin-systeem. Wil ik iets uitbreiden, dan vraag ik Claude om een paneel.
- Auto-update en notarisatie.
- Een chat-UI op de Agent SDK.
