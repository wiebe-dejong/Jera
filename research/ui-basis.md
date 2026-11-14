# UI-basis: pakketten (2026-11-04)

Bouwstenen voor de UI, per categorie. Dit kiest geen indeling en geen stijl.

Per pakket is gekeken naar:
- of het volledig te stylen is: het pakket heeft geen eigen CSS, of alle CSS is via variabelen en klassen te overschrijven;
- het gewicht, voor een M3 met 8 GB;
- of het werkt met React 19 in de Electron-renderer.

**Legenda:**
- ✓ = zelf gecontroleerd op 2026-11-04, via het npm-register, de GitHub-API, de broncode of een eigen meting.
- Zonder ✓ is het een claim uit de gelinkte bron.
- **Gewicht** is min+gzip van de volledige import, volgens Bundlephobia ✓. Met tree-shaking wordt het vaak kleiner.
- Alle frontendkandidaten accepteren React 19 als peer ✓. React zelf staat op 19.3.1 ✓.

## 1. Bestandsboom

| Pakket | Versie | Licentie | Gewicht | Lazy laden | Virtualisatie | Hernoemen | Naar buiten slepen | Te stylen |
|---|---|---|---|---|---|---|---|---|
| Headless Tree | 1.8.0, 15 okt 2026 ✓ | MIT ✓ | 11,5 + 1,1 kB ✓ | ✓ async loader met cache ✓ | met elke virtualizer ✓ | ✓ | ✓ `createForeignDragObject` ✓ | ja, headless |
| react-arborist | 4.0.1, 11 okt 2026 ✓ | MIT ✓ | 16,2 kB + react-dnd ✓ | zelf bouwen. Issue [#55](https://github.com/jameskerr/react-arborist/issues/55) staat open sinds 2022 ✓ | react-window, vaste `rowHeight` ✓ | ✓ | via react-dnd, niet getest | ja, eigen rij-component |
| react-complex-tree | 2.6.4, 24 aug 2026 ✓ | MIT ✓ | 15,7 kB ✓ | ✓ | nee, [#263](https://github.com/lukasbach/react-complex-tree/issues/263) staat open ✓ | ✓ | – | ja |
| React Aria `Tree` | 1.22.1, 9 okt 2026 ✓ | Apache-2.0 ✓ | onderdeel van 273 kB ✓ | `TreeLoadMoreItem` | niet gedocumenteerd voor Tree | nee | ✓ via MIME-typen | ja |

**Headless Tree:**
- Lazy laden, virtualisatie, hernoemen en naar buiten slepen zitten erin. Gecontroleerd in de README en de broncode ✓.
- Het is de opvolger van react-complex-tree, van dezelfde maker ✓. Versie 1.0 sinds april 2025 ✓.
- Geen afhankelijkheden ✓. Virtualiseren met `@tanstack/react-virtual` (MIT, 3.14.26, 3 nov 2026 ✓).
- **Gemeten** ✓ in Node 25 op deze Mac, alleen de core zonder React of DOM:

  | Open rijen | Volledige rebuild | Geheugen |
  |---|---|---|
  | 1.010 | 21 ms | 7 MB |
  | 10.010 | 120–150 ms | ~60 MB |
  | 30.030 | 270 ms | ~200 MB |
  | 100.100 | 1,4–1,7 s | ~670 MB |

  Dat is ongeveer 6 kB per open rij. De README claimt "100k+ items", maar dat gaat over renderen met virtualisatie, niet over geheugen.
- **Risico:** vrijwel één maker, met 677 commits. De andere bijdragers hebben er elk minder dan 30 ✓.

Zo werkt een bestandsboom ermee in Electron:
- **Lazy laden:** de data loader vraagt via IPC `fs.readdir` op in het main-proces. Meldt @parcel/watcher een wijziging, dan roep je `invalidateChildrenIds` aan op die map ✓.
- **Naar een terminal slepen:** `createForeignDragObject` zet het pad als `text/plain` in `dataTransfer` ✓. Een drop-handler op de terminal roept `term.paste(pad)` aan.
- **Uit de macOS Finder naar de cockpit:** `File.path` bestaat niet meer sinds Electron 32. Gebruik `webUtils.getPathForFile` ✓.
- **Uit de cockpit naar een andere app:** `webContents.startDrag` in het main-proces.
- **Contextmenu:** zit in geen van de bomen. Zie §3.

**react-arborist:** 3,7k sterren ✓. Hernoemen en slepen zitten erin. Verder:
- lazy laden moet je zelf bouwen ✓;
- slepen loopt via react-dnd, dus naast native HTML5-drag krijg je een tweede systeem;
- v4 is sinds 10 okt alleen ESM en vraagt Node ≥ 22.12 ✓.

## 2. Panelen, splits en tabs

Hieronder staat alleen wat de bouwstenen kunnen.

| Pakket | Versie | Licentie | Gewicht | Kan | Te stylen |
|---|---|---|---|---|---|
| react-resizable-panels | 4.18.0, 4 nov 2026 ✓ | MIT ✓ | 34,4 kB volledig. Tree-shakebaar sinds 4.15.3 ✓ | splits (`Group`, `Panel`, `Separator`). 2D-grids (`Grid`, sinds 4.14, 26 sep) ✓. `Workspace` (sinds 4.16, 31 okt) ✓: tabsets, tabs slepen, splitsen door op een rand te droppen, maximaliseren, een serialiseerbaar model | ja: geen eigen CSS, wel `classNames`, `styles` en `renderTab` ✓ |
| dockview | 8.8.3, 30 okt 2026 ✓ | MIT voor de core ✓. **`dockview-enterprise` is betaald** ✓ | 88–92 kB ✓ | docking-grid, tabs, slepen, zwevende groepen, popout-vensters, groepen aan een rand ✓, opslaan en herstellen | ja: CSS-variabelen `--dv-*`, een thema-object en eigen tabcomponenten ✓. `dockview.css` is wel nodig |
| FlexLayout | 0.11.1, 26 sep 2026 ✓ | MIT ✓ | 51,5 kB ✓ | tabsets, randtabsets met auto-hide, vastgezette tabs, tabgroepen, popouts, maximaliseren ✓ | ja: alle waarden als `--flexlayout-*` ✓. Een thema-CSS is nodig |
| allotment | 1.20.6, 12 okt 2026 ✓ | MIT ✓ | 9,5 kB ✓ | alleen splits | deels: eigen CSS met een paar variabelen |

**react-resizable-panels:**
- Eén pakket dekt alles, van één split tot tabsets die je versleept.
- Geen eigen CSS ✓.
- Veel gebruikt (~30 mln downloads per week ✓) en dagelijks onderhouden.
- `Workspace` was op 2026-11-04 4 dagen oud ✓. Niet getest.
- Een tab verplaatsen verplaatst zijn DOM, en een iframe herlaadt dan ✓. Een WebContentsView zit niet in de DOM en moet zelf meebewegen (risico 4 in `zelf-bouwen.md`).
- Inhoud van niet-zichtbare tabs kan gemount blijven (standaard), unmounten of in React `<Activity>` draaien ✓. Terminals laat je gemount.

**dockview:** geen afhankelijkheden ✓. Sinds juli 2026 is er een betaalde laag ✓. Daarin zitten onder meer vastgezette tabs, auto-hide van randgroepen, meerdere tabrijen, layoutgeschiedenis en docken met het toetsenbord ([licentiepagina](https://dockview.dev/docs/overview/licence)). Met render-modus `always` blijven verborgen panelen in de DOM, met `visibility: hidden` ✓.

**FlexLayout:** alles is gratis, ook auto-hide en vastgezette tabs. Wel versie 0.x en van één bedrijf (Caplin).

Prestaties zijn niet gemeten. Geen van de vier virtualiseert panelen, dus elk gemount paneel kost geheugen.

## 3. UI-primitieven

| Pakket | Versie | Licentie | Gewicht | Onderdelen | Opmerking |
|---|---|---|---|---|---|
| Base UI | 1.9.0, 9 okt 2026 ✓. 1.0 op 11 dec 2025 ✓ | MIT ✓ | 162 kB volledig ✓, per onderdeel minder | menu, contextmenu, menubalk, dialoog, alert-dialoog, popover, tooltip, toast, toolbar, tabs, combobox, autocomplete ✓ | Meer dan 100 commits sinds 1 okt ✓. shadcn/ui kiest het voor nieuwe projecten ([OpenReplay](https://blog.openreplay.com/shadcn-ui-radix-base-ui-switch/)) |
| Radix | 1.8.1, 12 okt 2026 ✓ | MIT ✓ | 13–30 kB per onderdeel ✓ | menu, contextmenu, menubalk, dialoog, popover, tooltip, toast, toolbar, tabs. Geen combobox ✓ | Nog actief ✓, maar er zijn zorgen over trager onderhoud ([DEV](https://dev.to/mashuktamim/is-your-shadcn-ui-project-at-risk-a-deep-dive-into-radixs-future-45ei)) |
| React Aria Components | 1.22.1, 9 okt 2026 ✓ | Apache-2.0 ✓ | 273 kB volledig ✓ | het breedst, ook Tree, Table en Virtualizer | Sterkste toegankelijkheid. Meer code per onderdeel |
| Ariakit | 0.4.42, 14 okt 2026 ✓ | MIT ✓ | 65 kB ✓ | menu, dialoog, popover, tooltip, combobox | Nog geen 1.0 ✓ |

Geen van de vier levert CSS. Je stylet via `className` en data-attributen, dus volledig met Tailwind ([Base UI: styling](https://base-ui.com/react/handbook/styling)).

**Base UI:** heeft een combobox en autocomplete, die Radix mist.

**Radix:** het kleinst per onderdeel, en cmdk gebruikt Radix Dialog al ✓.

### Native menu's of eigen menu's

| | Native Electron-menu | Eigen menu (Base UI) |
|---|---|---|
| Boven een WebContentsView | ja, macOS tekent het | nee: de view moet weg of kleiner |
| Te stylen | nee, het ziet eruit als macOS ✓ | volledig |
| Inhoud | tekst, icoon, vinkje, submenu, sneltoets. Op macOS 14+ ook kopjes, sublabels en badges ✓ | alles, ook statuskleur en zoeken |
| Toetsenbord, toegankelijkheid | van macOS | van Base UI |
| Code | `Menu.popup()` via IPC | een component in de renderer |

- De menubalk van macOS is er altijd en draagt de ⌘-sneltoetsen.
- Dialogen, popovers en tooltips in de renderer vallen ook onder een WebContentsView (zie de open punten).

## 4. Command palette

| Pakket | Versie | Licentie | Gewicht | Zoeken | Opmerking |
|---|---|---|---|---|---|
| cmdk | 1.1.1, 14 mrt 2025 ✓ | MIT ✓ | 16,3 kB ✓ | ingebouwd filter. Geen virtualisatie ([#335](https://github.com/dip/cmdk/issues/335)) ✓ | Zonder stijl. De laatste codewijziging was in aug 2025 ✓. ~45 mln downloads per week ✓ |
| kbar | 1.0.0, 10 aug 2026 ✓ | MIT ✓ | 26,9 kB ✓ | fuse.js 7, virtualisatie via `@tanstack/react-virtual` ✓ | Acties met id, sneltoets en ouder. Je rendert zelf |
| Base UI Autocomplete | in Base UI | MIT | – | filter, `virtualized`, een inline-modus | Geen palette-voorbeeld. Derden bouwen er wel een palette mee ([Kumo](https://kumo-ui.com/components/command-palette)) |

**cmdk:**
- Klein en zonder stijl.
- Er zijn hooguit honderden acties. Dat kan cmdk zonder virtualisatie.
- Snel een bestand openen: zet `shouldFilter={false}`, filter in het main-proces (bijvoorbeeld met `rg --files`) en toon de eerste ~100 resultaten.
- **Risico:** het onderhoud staat stil. De API is klein, dus vervangen kan.

**kbar:** virtualisatie en fuse.js zitten erin. Het actiemodel (id, sneltoets, ouder) past bij "commando's zijn de API".
- Eigen inschatting: de sneltoetsen van kbar luisteren in de DOM, dus ze zien geen toetsen in een WebContentsView. De sneltoetslaag hoort dus in het main-proces (`before-input-event` en menu-accelerators), welk pakket je ook kiest.

## 5. Iconen

### Mogen de IntelliJ-iconen in een eigen app?

Ja, met voorwaarden.
- ✓ `platform/icons/src/expui` in intellij-community bevat 820 iconen, plus `_dark`-varianten.
  - 776 dragen de kop "Use of this source code is governed by the Apache 2.0 license".
  - 40 hebben geen kop.
  - 4 hebben alleen "Copyright © JetBrains", waaronder het AI-icoon.
- ✓ `LICENSE.txt` in de repo heet sinds 15 juni 2026 "JetBrains Open-Source Build Terms 1.3". Die voorwaarden gaan over de gebouwde IDE's, en zeggen dat die uit Apache-2.0-software bestaan. De SVG's met kop blijven Apache-2.0.
- **Merkrecht:** Apache-2.0 geeft geen merkrecht (§6). JetBrains schreef dat productlogo's merken zijn en niet in een ander product mogen ([forum](https://intellij-support.jetbrains.com/hc/en-us/community/posts/206110389/comments/206104995)).
- Bestandstype-iconen met logo's van anderen (Docker, Vue, Jupyter) zijn merken van die eigenaren. Juist zulke bestanden hebben vaak geen licentiekop ✓.
- **Praktisch:** er is geen npm-pakket ✓. `@jetbrains/icons` is de webset van Ring UI, een andere set ✓. Je kopieert de SVG's uit de repo. De kleuren staan vast in elk bestand (`#6C707E`), met een apart `_dark`-bestand ✓. Voor eigen kleuren zet je ze om naar `currentColor`.
- **Voorwaarden:** gebruik alleen bestanden met de Apache-kop, geen product- of merklogo's, en lever LICENSE en NOTICE mee.

### Sets

| Set | Versie | Licentie | Aantal | Raster | Opmerking |
|---|---|---|---|---|---|
| Codicons | `latest` 0.0.46-24 (23 jul 2026), `next` 0.0.46-50 (2 nov 2026) ✓ | **CC-BY-4.0** voor de iconen, MIT voor de code ✓ | 657 ✓ | 16 px, `currentColor` ✓ | Git, PR (open, draft, closed, done), terminal, server, database, globe, debug en sync ✓. Als font of SVG |
| Tabler | 3.50.0, 27 okt 2026 ✓ | MIT ✓ | 6.257 ✓ | 24 px, lijndikte instelbaar | Ook 23 `file-type-*`-iconen en `git-branch-deleted` ✓. Outline en filled |
| Lucide | 1.68.0, 4 nov 2026 ✓ | ISC, plus MIT voor de delen uit Feather ✓ | 2.020 ✓ | 24 px, lijndikte instelbaar | Standaardset van shadcn/ui, dus veel gezien |
| IntelliJ (expui) | master ✓ | Apache-2.0 per bestand ✓ | ~776 bruikbaar ✓ | 16 px | Zie hierboven. Geen pakket |

**Codicons:**
- Getekend op 16 px ✓.
- **CC-BY-4.0** is geen MIT, Apache of ISC. Je moet de bron vermelden, met één regel in een "Over"- of licentiescherm.

**Tabler:** MIT en het grootst. Het raster is 24 px, dus op 16 px wordt elk icoon geschaald.

### Bestandstypen

| Set | Versie | Licentie | Aantal | Opmerking |
|---|---|---|---|---|
| material-icon-theme | 5.40.0, 26 okt 2026 ✓ | MIT ✓ | 1.256 SVG's ✓ | `generateManifest()` koppelt bestandsnamen en extensies aan iconen ✓. Gekleurd |
| vscode-icons | Iconify 1.2.96 ✓ | MIT ✓ | 1.719 ✓ | 32 px, gekleurd ✓. Het mappingpakket `vscode-icons-js` is van 2021 ✓ |
| file-icons | via Iconify | ISC ✓ | 930, 16 px, monochroom ✓ | Mapping niet gecontroleerd |

## 6. Git aansturen

| Aanpak | Versie | Licentie | Feiten |
|---|---|---|---|
| git-CLI direct (`execFile`) | systeem-git 2.39.5 (Apple) ✓ | git is GPL-2.0, maar wordt aangeroepen en niet meegeleverd | Zelfde config, hooks, worktrees en inloggegevens als bij Claude |
| simple-git | 4.0.2, 26 sep 2026 ✓ | MIT ✓ | Een wrapper om de CLI, met wachtrij, time-outs en parsers voor status en log |
| isomorphic-git | 1.44.10, 2 nov 2026 ✓ | MIT ✓ | Geen SSH ([#1419](https://github.com/isomorphic-git/isomorphic-git/issues/1419) staat open sinds 2021) ✓, en worktrees worden pas sinds maart 2026 gevonden ([#2293](https://github.com/isomorphic-git/isomorphic-git/pull/2293)) ✓ |
| dugite | 3.3.2, 26 okt 2026 ✓ | MIT ✓ | Levert een eigen git mee, zoals GitHub Desktop |
| nodegit | 0.27.0, juli 2020 ✓ | MIT ✓ | Stilgevallen |

**De git-CLI direct.** Commando's voor een branchoverzicht:
- `git for-each-ref` met `%(upstream:track)`, dat `[gone]` geeft als de remote branch weg is.
- `%(ahead-behind:main)` geeft voor en achter op main in één aanroep. Dat kan pas sinds git 2.41 ✓, en de Apple-git op deze Mac is 2.39 ✓. Het kan met Homebrew-git, of per branch met `git rev-list --left-right --count main...<branch>`.
- `git worktree list --porcelain`, `git status --porcelain=v2 -z` en `git fetch --prune`.

Eigen inschatting: simple-git parseert juist deze uitvoer niet, dus je zou toch `raw()` gebruiken. Een time-out zit al in `execFile`.

**simple-git 4** heeft een wachtrij met een maximum aan gelijktijdige processen. In v4 ✓:
- er is alleen nog een named export;
- het filtert onveilige omgevingsvariabelen, die je expliciet moet toestaan.

Controleer dan dat de SSH- en GH-variabelen doorkomen.

### PR-status via `gh`

- **Eén aanroep per repo** ✓: `gh pr list --state all --json number,state,headRefName,headRefOid,mergedAt,baseRefName --limit 200`.
- **Een squash-merge herkennen:**
  - de PR is merged;
  - de lokale tip is gelijk aan `headRefOid`, dus er zijn na de merge geen commits bijgekomen.
- **Meerdere GitHub-accounts:** `gh auth token --user <naam>` geeft het token van één account ✓. Geef dat per project mee als `GH_TOKEN`.
- **Versie:** lokaal staat gh 2.93.0, de nieuwste is 2.103.3 (26 okt 2026) ✓. MIT.
- **Zonder gh:** de truc van [git-delete-squashed](https://github.com/not-an-aardvark/git-delete-squashed) (`commit-tree` plus `cherry`) ✓. Die maakt wel tijdelijke losse commits aan.
- **PATH:** een app die je vanuit de Finder start, krijgt niet het PATH van je shell. launchd heeft hier geen eigen PATH ✓, dus `/opt/homebrew/bin/gh` wordt dan niet gevonden. Gebruik `shell-env` (MIT, 4.0.3 ✓) of zet vaste paden in de config.

## 7. Lettertypen

| Lettertype | Versie | Licentie | Beschikbaar |
|---|---|---|---|
| Systeemfont (SF Pro) | met macOS | Apple: niet meeleveren ([Apple Fonts](https://developer.apple.com/fonts/)) | `system-ui` of `-apple-system` in Chromium |
| Inter | 4.1, 16 nov 2024 ✓ | OFL-1.1 ✓ | `@fontsource-variable/inter` 5.3.0, 1,9 MB uitgepakt ✓ |
| JetBrains Mono | 2.304, 14 jan 2023 ✓ | OFL-1.1 ✓ | `@fontsource-variable/jetbrains-mono` 5.3.0, 0,2 MB ✓ |
| SF Mono | met macOS: `SFNSMono.ttf` in `/System/Library/Fonts` ✓ | Apple: niet meeleveren | Chromium kent `ui-monospace` niet ([caniuse](https://caniuse.com/extended-system-fonts)). Opvragen bij naam is niet getest |
| Menlo | met macOS ✓ | Apple | bij naam |

OFL-1.1 mag je meeleveren en insluiten. Je mag het lettertype alleen niet los verkopen. Van Inter en JetBrains Mono is al een tijd geen nieuwe release verschenen, maar beide zijn stabiel.

## Open punten

1. **Overlays en de browser:** een WebContentsView ligt boven de DOM. De palette, dialogen, popovers en eigen menu's verdwijnen eronder. Mogelijkheden: de view verbergen of verkleinen, of overlays in een eigen view erboven tekenen.
2. **`Workspace` van react-resizable-panels is nieuw** en niet getest.
3. **Git-versie:** `%(ahead-behind)` vraagt git ≥ 2.41. De Apple-git is 2.39.
4. **PATH** voor git en gh in het main-proces.
5. **Projecten en GitHub-accounts:** `GH_TOKEN` vraagt per project een vastgelegd account.
6. **Geheugen van de boom:** Headless Tree kost ~6 kB per open rij, dus ~200 MB bij 30.000 open rijen. react-arborist is niet gemeten.
7. **SF Mono bij naam** in Electron is niet getest.

## Bronnen

- **Gecontroleerd via:** het [npm-register](https://registry.npmjs.org/), de GitHub-API (`gh api`), [Bundlephobia](https://bundlephobia.com) en [Iconify](https://api.iconify.design).
- **Bestandsboom:** https://github.com/lukasbach/headless-tree · https://headless-tree.lukasbach.com · https://github.com/lukasbach/headless-tree/blob/main/packages/sb-react/src/dnd/drag-outside.stories.tsx · https://github.com/jameskerr/react-arborist · https://github.com/jameskerr/react-arborist/releases/tag/v4.0.0 · https://github.com/lukasbach/react-complex-tree · https://react-aria.adobe.com/Tree · https://react-aria.adobe.com/Virtualizer · https://github.com/TanStack/virtual
- **Electron:** https://www.electronjs.org/docs/latest/breaking-changes · https://www.electronjs.org/docs/latest/api/web-utils · https://www.electronjs.org/docs/latest/tutorial/native-file-drag-drop · https://www.electronjs.org/docs/latest/api/menu · https://www.electronjs.org/docs/latest/api/menu-item
- **Panelen:** https://github.com/bvaughn/react-resizable-panels · https://github.com/bvaughn/react-resizable-panels/releases · https://dockview.dev/docs/overview/licence · https://github.com/dockview/dockview/blob/master/packages/docs/docs/core/panels/rendering.mdx · https://github.com/dockview/dockview/blob/master/packages/docs/docs/core/theming.mdx · https://github.com/caplin/FlexLayout · https://github.com/johnwalley/allotment
- **Primitieven:** https://base-ui.com · https://github.com/mui/base-ui · https://github.com/radix-ui/primitives · https://react-aria.adobe.com · https://github.com/ariakit/ariakit · https://blog.openreplay.com/shadcn-ui-radix-base-ui-switch/ · https://dev.to/mashuktamim/is-your-shadcn-ui-project-at-risk-a-deep-dive-into-radixs-future-45ei
- **Command palette:** https://github.com/dip/cmdk · https://github.com/timc1/kbar/releases/tag/v1.0.0 · https://base-ui.com/react/components/autocomplete · https://kumo-ui.com/components/command-palette
- **Iconen:** https://github.com/JetBrains/intellij-community/tree/master/platform/icons/src/expui · https://github.com/JetBrains/intellij-community/blob/master/LICENSE.txt · https://www.apache.org/licenses/LICENSE-2.0 · https://intellij-support.jetbrains.com/hc/en-us/community/posts/206110389/comments/206104995 · https://intellij-support.jetbrains.com/hc/en-us/community/posts/206196969/comments/206371385 · https://jetbrains.com/company/brand · https://github.com/microsoft/vscode-codicons · https://github.com/tabler/tabler-icons · https://github.com/lucide-icons/lucide · https://github.com/material-extensions/vscode-material-icon-theme · https://github.com/vscode-icons/vscode-icons
- **Git:** https://github.com/steveukx/git-js/releases · https://git-scm.com/docs/git-for-each-ref · https://github.com/isomorphic-git/isomorphic-git · https://github.com/desktop/dugite · https://cli.github.com/manual/gh_pr_list · https://cli.github.com/manual/gh_auth_token · https://github.com/not-an-aardvark/git-delete-squashed · https://github.com/sindresorhus/shell-env
- **Lettertypen:** https://github.com/rsms/inter/releases · https://github.com/JetBrains/JetBrainsMono/releases · https://fontsource.org · https://developer.apple.com/fonts/ · https://caniuse.com/extended-system-fonts
