# Viewers: pakketten (2026-11-04)

Per pakket is gekeken naar: hoe goed het werkt, het gewicht voor een M3 met 8 GB, en of de stijl volledig aan te passen is. De bestandsboom staat in `ui-basis.md`.

## Hoe gemeten

- ✓ = zelf gecontroleerd. (b) = claim uit de gelinkte bron, niet zelf getest.
- **Versies en datums:** opgehaald uit de npm-registry en de GitHub-API op 2026-11-04.
- **Bundelgrootte:** esbuild 0.28, geminificeerd, met React erbuiten. "gz" is gzip. Bij pakketten die delen pas laden wanneer ze nodig zijn (lazy chunks) staat de eerste lading apart.
- **Geheugen en tijd:**
  - in de renderer van Electron 45.2.2 op deze M3 met 8 GB;
  - JS-heap via `performance.memory`, procesgeheugen via `app.getAppMetrics()`;
  - er liepen tegelijk andere Claude-sessies (load ~15). Het procesgeheugen schommelde daardoor ±100 MB per run. De JS-heap was stabiel.
- **Testbestanden:**
  - CSV met 100k rijen × 10 kolommen (11,8 MB);
  - dezelfde data als xlsx, met opmaak en een samengevoegde cel (6 MB);
  - de PDF 32000-spec (756 pagina's, 22 MB);
  - een JS-bestand van 88k regels (2,6 MB).
- Leeg Electron-venster: ~130 MB voor alle processen samen ✓.

## 1. Markdown

| Pakket | Versie | Licentie | Gewicht ✓ | Te stylen |
|---|---|---|---|---|
| react-markdown + remark-gfm | 10.1.0 (2025-03-07) + 4.0.1 (2025-02-10) | MIT | 160 KB min / 49 KB gz | Ja, geen eigen CSS |
| marked | 18.1.3 (2026-11-02) | MIT | 36 KB / 12 KB | Ja, maar levert een HTML-string op |
| streamdown (Vercel) | 2.7.1 (2026-10-19) | Apache-2.0 | 525 KB / 157 KB | Deels: komt met eigen Tailwind-classes |
| mermaid | 12.3.0 (2026-11-03) | MIT | 913 KB / 189 KB eerste lading, 5,1 MB in totaal | Via `theme: 'base'` en `themeVariables` (b) |

- **react-markdown:**
  - GFM (tabellen, takenlijsten) komt via remark-gfm;
  - elk element (`img`, `code`, `table`) is te vervangen door een eigen component (b);
  - de laatste release is van maart 2025, maar de repo is actief (push 2026-10-03, 5 open issues) ✓. Het pakket is af, niet verlaten.
- **Relatieve afbeeldingen:**
  - `urlTransform` laat standaard relatieve URL's door en blokkeert alleen onveilige protocollen ✓ (in de broncode gezien);
  - de renderer moet `./img/x.png` zelf oplossen ten opzichte van de map van het md-bestand;
  - serveer zo'n bestand via een eigen protocol (`protocol.handle`) en niet via `file://`.
- **Rauwe HTML:** staat standaard uit, en dat is veilig. Is HTML in md nodig, gebruik dan `rehype-raw` met `rehype-sanitize`.
- **Codeblokken:**
  - `highlightCode` met `classHighlighter` uit `@lezer/highlight` ✓ (aanwezig in 1.2.5) gebruikt dezelfde parsers en CSS-classes als CodeMirror en @codemirror/merge, dus één kleurenschema voor alle drie;
  - react-markdown + remark-gfm + `@lezer/highlight` samen: 185 KB min / 57 KB gz ✓, plus een parser per taal die pas laadt als die taal voorkomt;
  - Shiki kleurt beter (TextMate-grammatica's), maar brengt een tweede thema-systeem mee. Fijnmazig met de JS-engine en 2 talen: 418 KB / 80 KB ✓.
- **Mermaid:**
  - laad het met `import()` en alleen als er een mermaid-blok in het bestand staat;
  - gebruik `securityLevel: 'strict'` (b).
- **marked:**
  - klein en snel;
  - maar het resultaat is een HTML-string. Je hebt dan DOMPurify en `dangerouslySetInnerHTML` nodig, en geen React-component per element.

## 2. PDF

| Optie | Versie | Licentie | Gewicht | Te stylen |
|---|---|---|---|---|
| pdfjs-dist + `PDFViewer` | 6.5.438 (2026-11-01) | Apache-2.0 | 436 KB + viewer 168 KB min; worker 1,2 MB in een aparte thread ✓ | Ja |
| react-pdf (wojtekmaj) | 11.0.0 (2026-09-10) | MIT | 621 KB / 188 KB, met pdf.js erin ✓ | Ja |
| EmbedPDF (PDFium als WASM) | 2.15.1 (2026-09-16); 3.0.0-next.21 (2026-11-03) | MIT op npm; de repo zegt Apache-2.0 ✓ | WASM 4,6 MB; kant-en-klare viewer 1,1 MB JS ✓ | Ja, met headless plugins (b) |
| Ingebouwd in Chromium | Electron 45.2.2 | (Chromium) | Geen code | Nee |

Zelf getest in Electron 45.2.2 ✓:
- **Ingebouwd:**
  - werkt in een BrowserWindow, in een WebContentsView en in een iframe;
  - werkt ook met `plugins: false` (de screenshot was identiek);
  - toont de eigen donkere balk van Chrome en een kolom met miniaturen;
  - `#toolbar=0` haalt de balk weg, maar de donkergrijze achtergrond rond de pagina blijft;
  - de docs van Electron noemen geen API om vanuit de app te zoeken of naar een pagina te gaan ✓;
  - het kost 2 extra processen (6 tegen 4).
- **pdf.js:**
  - de spec van 756 pagina's opent op paginabreedte;
  - de achtergrond en de paginarand zijn eigen CSS;
  - de tekstkwaliteit zag er op deze PDF hetzelfde uit als bij de ingebouwde viewer.
- **Geheugen voor alle processen samen:**
  - pdf.js: 159 en 249 MB;
  - ingebouwd in een iframe: 199 en 424 MB;
  - een leeg venster: 129 MB.
  - pdf.js was in deze test dus niet zwaarder. De spreiding is groot door de load.

Per optie:
- **pdf.js:**
  - de eigen `PDFViewer` (uit `pdfjs-dist/web/pdf_viewer.mjs`) rendert alleen de zichtbare pagina's en kan zoeken (`PDFFindController`) en links volgen (b);
  - een balk bouw je zelf in React, dus hij past bij elke stijl;
  - `pdf_viewer.css` is nodig voor de tekstlaag, en de rest overschrijf je.
- **react-pdf:** een dunne React-schil. De README noemt geen virtualisatie of zoeken ✓. Voor lange PDF's bouw je dat er zelf omheen.
- **EmbedPDF:**
  - de render is van PDFium, net als in Chrome, en de opzet is headless (b);
  - maar v3 is nog prerelease;
  - de WASM laadt standaard van cdn.jsdelivr.net ✓ (URL in de code gevonden). Die moet je zelf meeleveren;
  - het bedrijf verkoopt een server (CloudPDF, Fair Source). De bibliotheken zijn Apache-2.0 ✓.
- **Ingebouwd:** weergave zonder code, niet te stylen.
- **Beveiliging:** pdf.js had twee lekken waarbij een PDF JavaScript kon uitvoeren: CVE-2024-4367 en CVE-2026-16633 (opgelost in 6.2.108) ✓. Houd pdf.js bij en zet `isEvalSupported: false`.

## 3. CSV

| Pakket | Versie | Licentie | Gewicht ✓ | Te stylen |
|---|---|---|---|---|
| papaparse | 5.7.0 (2026-08-24) | MIT | 19 KB / 7 KB | n.v.t. |
| d3-dsv | 3.0.2 (2026-11-03) | ISC | 2 KB / 1 KB | n.v.t. |
| react-data-grid (Comcast) | 7.0.0-beta.63 (2026-10-25) | MIT | 35 KB / 13 KB | Ja: CSS-variabelen `--rdg-*` |
| @tanstack/react-virtual | 3.14.26 (2026-11-03) | MIT | 34 KB / 10 KB | Ja, volledig headless |
| Glide Data Grid | 6.0.3 (2024-02-03) | MIT | 289 KB / 98 KB | ⚠ Canvas: alleen via een thema-object, niet via CSS |
| AG Grid Community | 36.3.0 (2026-11-04) | MIT; ⚠ Enterprise is betaald | 727 KB–1,2 MB min | Via een Theming API, met een eigen basislook |

- **Parsen ✓:**
  - papaparse doet 100k rijen in 0,2–0,5 s, met ~60 MB JS-heap (in Node en Electron gelijk);
  - d3-dsv is even snel, maar kan niet streamen en raadt het scheidingsteken niet (b);
  - papaparse kan in een worker draaien en in stukken streamen (b). Doe dat bij grote bestanden, dan blijft de UI vloeiend.
- **react-data-grid ✓:**
  - 100k rijen in Electron: ~500 DOM-nodes en 61 MB heap, inclusief de data;
  - rijhoogte 22 px gaat zonder problemen;
  - virtualiseert rijen en kolommen. Ook vaste kolommen, kolommen samenvoegen, breedte slepen, sorteren, toetsenbord en kopiëren (b);
  - vraagt React ^19.2 ✓;
  - ⚠ v7 heet al jaren "beta", maar is actief (push op 2026-11-04) ✓. Zet de versie vast.
- **TanStack Virtual:** helemaal headless. De tabel, de vaste kopregel en de kolombreedtes bouw je zelf (~100 regels). Ook gemeten: ~500 DOM-nodes bij 100k rijen ✓.
- **Glide:**
  - tekent op canvas, dus snel bij miljoenen rijen (b);
  - de stabiele 6.0.3 ondersteunt geen React 19 (peer `react-dom` tot en met 18) ✓. Dat kan alleen met een alpha (6.0.4-alpha24, okt 2025).
- **AG Grid:** het zwaarst van de grids, met een eigen basislook.

## 4. Xlsx

| Optie | Versie | Licentie | Gewicht ✓ | 100k rijen in Electron ✓ |
|---|---|---|---|---|
| SheetJS CE | 0.20.3 (tag 2024-07-18; commits tot 2026-02-09) | Apache-2.0; ⚠ Pro is betaald | 362 KB / 121 KB | 1,6–1,7 s |
| hucre | 1.3.0 (2026-10-26); eerste release 2026-03-27 | MIT | 136 KB / 41 KB | 0,8–0,9 s |
| read-excel-file | 9.3.14 (2026-10-29) | MIT | 52 KB / 16 KB | 0,9–1,0 s |
| exceljs | 4.4.0 (2023-10-19) | MIT | 925 KB / 264 KB | (in Node: 3,5–5 s) |
| Quick Look (`win.previewFile`) | macOS | (systeem) | Geen code | Opent direct |

- **SheetJS CE ✓:**
  - ⚠ de versie op npm (`xlsx` 0.18.5, uit 2022) heeft CVE-2023-30533 en CVE-2024-22363 (beide hoog). Volgens GitHub is er op npm geen oplossing;
  - installeer daarom de tarball van cdn.sheetjs.com. Dat werkte met `npm install <url>`;
  - Dependabot ziet dan geen updates, maar er komen ook weinig releases;
  - leest geformatteerde waarden, samengevoegde cellen en met `cellStyles` ook vulkleuren;
  - volledige opmaak zit in de betaalde Pro-versie (b);
  - leest ook xls, xlsb en ods (b).
- **hucre ✓:**
  - het snelst en het zuinigst: 36–45 MB heap, tegen 144–184 MB voor SheetJS (in Node);
  - leest opmaak en samengevoegde cellen, en heeft geen dependencies;
  - ⚠ wel pas 7 maanden oud en het werk van één maintainer.
- **exceljs:** ⚠ staat stil (geen release sinds okt 2023, laatste push jan 2025, 824 open issues) ✓. Het kost ook het meeste geheugen: 350–390 MB heap ✓.
- **read-excel-file:** licht, maar leest alleen waarden (b).
- **Quick Look:**
  - `previewFile` opent het Quick Look-paneel van macOS ✓ (Electron-docs);
  - een miniatuur via `qlmanage` toonde vet, vulkleur en een decimale komma ✓. Op deze Mac staat Excel, dus het is niet zeker of de weergave van macOS of van Excel komt;
  - nadelen: het is een los paneel buiten de eigen viewer, met een vaste look, zonder zoeken of kopiëren vanuit de app, en alleen voor macOS.
- **Hetzelfde grid als CSV:** ja. Beide leveren rijen met tekst voor react-data-grid. Xlsx heeft er nog drie dingen bij:
  - een tab per sheet;
  - samengevoegde cellen via `colSpan`. ⚠ De README noemt geen samenvoeging over rijen;
  - opmaak per cel via `cellClass` of een eigen `renderCell`.

## 5. Afbeeldingen

| Pakket | Versie | Licentie | Gewicht ✓ | Te stylen |
|---|---|---|---|---|
| react-zoom-pan-pinch | 4.2.0 (2026-09-03) | MIT | 47 KB / 14 KB | Ja, geen eigen UI |
| @panzoom/panzoom | 4.6.2 (2026-04-02) | MIT | 9 KB / 4 KB | Ja, geen eigen UI |
| OpenSeadragon | 6.1.1 (2026-09-09) | BSD-3-Clause | groot | Deels; bedoeld voor deep-zoom-tegels |
| yet-another-react-lightbox | 3.33.0 (2026-10-30) | MIT | – | ⚠ Lightbox met een eigen look |

- **react-zoom-pan-pinch ✓:**
  - een SVG in `<img>`, ingezoomd tot 8×, bleef scherp in Electron;
  - zet geen `will-change: transform`, wat SVG wazig zou maken;
  - werkt met React 19 (peer `*`).
- **@panzoom/panzoom:** zonder framework, dus je zet het in een `useEffect`. Kan ook SVG-elementen zelf zoomen (b).
- **SVG:** toon het als `<img>`. Dan draaien scripts in de SVG niet (b).
- **Grote rasterbestanden:** gedecodeerd kosten die breedte × hoogte × 4 bytes. Een foto van 8000 × 8000 is 256 MB. Overweeg een limiet of een verkleinde versie.
- **OpenSeadragon:** voor tegels van gigapixel-beelden.

## 6. Code bekijken

| Pakket | Versie | Licentie | Gewicht ✓ | 88k regels in Electron ✓ | Te stylen |
|---|---|---|---|---|---|
| CodeMirror 6 | view 6.43.15 (2026-10-19) | MIT | 424 KB / 147 KB, met 2 talen | 95 ms, ~300 DOM-nodes, 18 MB heap | Ja, volledig |
| Monaco | 0.57.0 (2026-09-24) | MIT | 2,7 MB / 0,7 MB, plus 1,8 MB CSS en een worker van 300 KB | 216 ms, 78 MB heap | ⚠ Kleuren wel; de widgets houden de look van VS Code (b) |
| Shiki | 4.6.0 (2026-11-02) | MIT | 418 KB / 80 KB (fijnmazig) | 29k regels: 4,4 s, 5,6 MB HTML, 149k spans (in Node) | Ja |

- **CodeMirror 6:**
  - rendert alleen wat zichtbaar is, dus grote bestanden zijn geen probleem ✓;
  - talen laden pas als ze nodig zijn via `@codemirror/language-data` (124 losse chunks, samen 1,5 MB) ✓. Python, TS, SQL, YAML en TOML zitten erin ✓;
  - zelfde basis als @codemirror/merge;
  - het thema regel je met `EditorView.theme` en een `HighlightStyle` of `classHighlighter` (b);
  - de ontwikkeling verhuisde naar code.haverbeke.berlin. De GitHub-repo is sinds 2026-04-15 gearchiveerd ✓, maar er verschijnen nog steeds releases ✓. Dus niet verlaten.
- **Monaco:**
  - ruim 4× zoveel heap en 6× zoveel JS;
  - je moet workers instellen;
  - `@monaco-editor/react` laadt Monaco standaard van een CDN (b).
- **Shiki:**
  - de mooiste kleuring;
  - maar zet het hele bestand in de DOM zonder virtualisatie ✓. Dus alleen geschikt voor kleine bestanden en voor md-codeblokken.

## 7. Diff

| Pakket | Versie | Licentie | Gewicht ✓ | Te stylen |
|---|---|---|---|---|
| @codemirror/merge | 6.12.3 (2026-10-19) | MIT | +28 KB / +10 KB bovenop CodeMirror | Ja, hetzelfde thema als de codeviewer |
| @pierre/diffs | 1.6.4 (2026-11-04) | Apache-2.0 | 58 KB / 21 KB, plus Shiki met grammatica's (11 MB aan lazy chunks) | Ja: ~100 CSS-variabelen `--diffs-*` en `unsafeCSS` ✓ |
| react-diff-view | 3.3.3 (2026-03-30) | MIT | 53 KB / 18 KB | Ja, eigen CSS te vervangen (b) |
| diff2html | 3.4.56 (2026-01-31) | MIT | 44 KB / 13 KB | ⚠ Een HTML-string met een vaste look; wel te overschrijven |
| @git-diff-view/react | 0.1.8 (2026-10-15) | MIT | 1,2 MB / 337 KB | Ja; nog voor 1.0 |
| Monaco diff | 0.57.0 | MIT | zoals Monaco | ⚠ zoals Monaco |

- **@codemirror/merge ✓ (in de types gezien):**
  - `MergeView` toont naast elkaar, `unifiedMergeView` inline;
  - `collapseUnchanged`, `highlightChanges` en knoppen om een wijziging terug te draaien zitten erin;
  - `scanLimit` en `timeout` houden grote diffs binnen de perken;
  - het neemt twee volledige teksten, geen patch. De oude versie komt uit `git show HEAD:pad`;
  - de knoppen per wijziging passen bij hunks stagen via `git apply --cached`.
- **@pierre/diffs:**
  - gemaakt voor reviews van veel bestanden: Shadow DOM, CSS Grid, virtualisatie (b);
  - neemt een patch of twee bestandsversies aan ✓ (README);
  - ⚠ jong en snel in beweging: eerste release dec 2025, 112 versies ✓;
  - het gebruikt Shiki, dus een tweede kleurensysteem naast CodeMirror.
- **react-diff-view en diff2html:** lezen een unified diff. Licht, maar de kleuring moet je er apart bij zetten.

## Open punten

1. **Kleuring.** Lezer en Shiki zijn niet vergeleken op kwaliteit per taal.
2. **PDF.** pdf.js is alleen getest op de PDF 32000-spec, niet op formulieren, scans of afwijkende fonts. EmbedPDF v3 is nog prerelease.
3. **Xlsx-opmaak.** SheetJS en hucre zijn alleen getest op het testbestand, niet op andere Excel-bestanden.
4. **Samengevoegde rijen.** react-data-grid noemt alleen samenvoegen over kolommen. Controleer hoe xlsx-bestanden met samengevoegde rijen eruitzien.
5. **Distributie van SheetJS.** Zet de tarball-URL vast, of neem de tgz op in de repo. Volg updates via de RSS van cdn.sheetjs.com.
6. **Grote bestanden.** Parse CSV en xlsx in een worker of utility-proces. Leg een grens vast (bijvoorbeeld boven 1M rijen of 100 MB), en toon daarboven alleen een voorbeeld.
7. **Beveiliging.**
   - Geen rauwe HTML in md.
   - Mermaid op `strict`.
   - pdf.js bijhouden.
   - Lokale bestanden via een eigen protocol dat alleen bestanden leest.
   - Renderers met `contextIsolation` aan en zonder `nodeIntegration`.
8. **Meting.** Het procesgeheugen is gemeten op een belaste machine. Meet het opnieuw als er weinig draait, zodra er een build is.

## Bronnen

- **Markdown:** https://github.com/remarkjs/react-markdown · https://github.com/remarkjs/remark-gfm · https://github.com/markedjs/marked · https://github.com/vercel/streamdown · https://lezer.codemirror.net/docs/ref/#highlight.highlightCode · https://shiki.style/guide/bundles · https://mermaid.js.org/config/theming.html · https://www.electronjs.org/docs/latest/api/protocol
- **PDF:** https://github.com/mozilla/pdf.js · https://www.npmjs.com/package/pdfjs-dist · https://github.com/wojtekmaj/react-pdf · https://github.com/embedpdf/embed-pdf-viewer · https://github.com/embedpdf/embed-pdf-viewer/blob/main/LICENSING.md · https://www.electronjs.org/docs/latest/api/structures/web-preferences · https://github.com/advisories/GHSA-wgrm-67xf-hhpq · https://github.com/advisories/GHSA-hq66-cqwq-w95j · testbestand: https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/PDF32000_2008.pdf
- **CSV:** https://www.papaparse.com/docs · https://github.com/d3/d3-dsv · https://github.com/Comcast/react-data-grid · https://tanstack.com/virtual · https://github.com/glideapps/glide-data-grid · https://www.ag-grid.com/license-pricing/
- **Xlsx:** https://cdn.sheetjs.com/ · https://git.sheetjs.com/sheetjs/sheetjs · https://docs.sheetjs.com/docs/getting-started/installation/nodejs · https://github.com/advisories/GHSA-4r6h-8v6p-xvw6 · https://github.com/advisories/GHSA-5pgg-2g8v-p4x9 · https://github.com/productdevbook/hucre · https://github.com/exceljs/exceljs · https://gitlab.com/catamphetamine/read-excel-file · https://www.electronjs.org/docs/latest/api/base-window
- **Afbeeldingen:** https://github.com/BetterTyped/react-zoom-pan-pinch · https://github.com/timmywil/panzoom · https://github.com/openseadragon/openseadragon · https://github.com/igordanchenko/yet-another-react-lightbox
- **Code:** https://codemirror.net/ · https://code.haverbeke.berlin/codemirror/dev · https://github.com/codemirror/dev · https://github.com/microsoft/monaco-editor · https://github.com/suren-atoyan/monaco-react · https://github.com/shikijs/shiki
- **Diff:** https://codemirror.net/docs/ref/#merge · https://diffs.com · https://github.com/pierrecomputer/pierre · https://github.com/otakustay/react-diff-view · https://github.com/rtfpessoa/diff2html · https://github.com/MrWangJustToDo/git-diff-view
