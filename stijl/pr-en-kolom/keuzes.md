# PR en kolom: keuzes

Bord: https://claude.ai/artifact/WkjjYE5eSDJeTbDx8nPbih · bron: `keuzebord.html`. Gaat over `pijl()` en `teken_branch()` in `app/src/gitpaneel.rs` (het teken op de merge) en over `kolom()` in `app/src/venster.rs` (de linkerkolom).

## Stand na ronde 2 (2026-11-20)

Open 0 · deels 0 · later 0 · vast 2. Gebouwd in `app/src/gitpaneel.rs` en `app/src/venster.rs`.

## 01 Teken op de merge · C: Ruit

| | |
|---|---|
| Op de lijn | ruit 13 px van punt tot punt, in plaats van het bolletje |
| Op het eindpunt | 8 px, in het vierkant |
| Kleur | de branch die binnenkomt |
| Bocht en lijnen | als nu |

## 02 Linkerkolom en sessie · B: Eigen vlak

| | |
|---|---|
| Kolom | bar, zoals de titelbalk |
| Lijn | 1 px, line, als nu |
| Gekozen rij | sel op bar |
| Design system | de regel "het venster is één vlak in bg" krijgt een uitzondering voor de kolom |
| Contrast | ink op bar 12.7:1 donker, 13.5:1 licht · mute op bar 5.8:1 donker, 5.3:1 licht |

## Afgevallen

- 01: A Dwarsstreep (15 × 3 px dwars), B Blokje (9 × 9 px).
- 02: A Zware lijn (3 px, line-strong), C Goot (8 px bar tussen 2 lijnen).
