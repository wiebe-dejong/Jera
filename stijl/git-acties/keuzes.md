# Git-paneel acties: keuzes

Bord: https://claude.ai/artifact/1niZcnFTnXRY6A8Rp5Wn1Q · bron: `keuzebord.html`. Gaat over `app/src/gitpaneel.rs` en `app/src/git.rs`.

## Stand na ronde 5 (2026-11-15)

Open 0 · deels 0 · later 0 · vast 10. Nog te bouwen.

Leidraad: alleen tonen wat de graaf en de actieregel niet al zeggen. Een PR maak ik via Claude, niet in Jera.

## 01 Plek van de acties · A: Regel bovenaan

| | |
|---|---|
| Plek | regel boven de graaf, 34 px, lijn eronder |
| Toont | commit n · update ↓n · push ↑n |
| Niets te doen | geen regel |
| Push | pas als er niets te halen is |

## 02 Commitbericht · B: Voorstel van Claude

| | |
|---|---|
| Veld | voorstel uit de diff, `claude -p`, ± 1 s |
| Enter | commit alle gewijzigde bestanden |
| Typen | past het voorstel aan |
| Escape | terug |

## 03 Update bij nieuw aan beide kanten · B: Rebase

| | |
|---|---|
| git | `pull --rebase` |
| Jouw commits | bovenop die van GitHub |
| Daarna | push ↑n |
| Conflict | stoppen, vraag Claude |

## 04 Ophalen · A: Vanzelf

| | |
|---|---|
| Wanneer | bij openen van het paneel en elke 5 min |
| opgehaald | alleen het tijdstip |
| Klik | niets |

## 05 Onder de muis: aanwijzen · B: Andere lijn zwakker

| | |
|---|---|
| Andere lijn | 30% |
| Punt | kader 1 px ink, 18 px |
| Raakvlak | het dichtstbijzijnde punt binnen 30 px opzij en 20 px boven en onder |
| Loslaten | na 150 ms |

## 06 Onder de muis: tekst · A: Geen

| | |
|---|---|
| Tekst | geen |

## 07 Klikken: gekozen en details · B: Kaartje blijft staan

| | |
|---|---|
| Plek | naast het punt, zo breed als de tekst, hoogstens 290 px |
| Kaartje | bg, rand line-strong, streep accent 2 px links |
| Weg | Escape, klik op leeg vlak, nog een klik op het punt |

## 08 Detail van een branch · G: Alleen achter op main

| | |
|---|---|
| Toont | naam; n achter op main |
| Niet achter | alleen de naam |
| GitHub | staat in de actieregel |

## 09 Detail van een commit · G: Wie en hoe laat

| | |
|---|---|
| Regel | auteur · tijdstip |
| Andere dag | datum erbij |
| Titel, bestanden | niet |

## 10 Meer dan commit, push, update · A: Niets extra

| | |
|---|---|
| Acties | commit, push, update |
| PR | via Claude |
