# Server rechtsonder: keuzes

Bord: https://claude.ai/artifact/AjZ1hqvzhSfY1fJhWCLvuo · bron: `keuzebord.html`. Gaat over `app/src/venster/poortenpaneel.rs` en `app/src/venster.rs`: poorten, browsers en artifacts gaan van het paneel rechts (onder git) naar rechtsonder in het venster.

Stand na ronde 3 (2026-11-15): 0 open, 0 deels, 0 later, 4 vast. Gebouwd; in het design system als voorstel Statusbalk.

## Vast zonder bord

- Een teller per soort: poorten, browsers en artifacts. Een soort met 0 staat er niet; staan alle 3 op 0, dan staat er niets.
- Het icoon achter de titel van een sessie met een poort of browser is weg.

## 01 Plek rechtsonder · A: Statusbalk onderaan

| | |
|---|---|
| Plek | rij onderaan rechts van de linkerkolom, op één lijn met de kop oud |
| Breedte | terminal en kolom rechts |
| Hoogte | h-rij (26 px), op bg, lijn erboven |
| Links | leeg |
| Open | streep accent boven |
| Alle 3 op 0 | rij blijft staan, leeg |
| Na de bouw | eerst 24 px op bar onder het hele venster; daarna gelijk gemaakt aan de rij oud |

## 02 Teller · A: Woord en aantal

| | |
|---|---|
| Per soort | poorten 3 · browsers 1 · artifacts 3 |
| Woord | mute |
| Aantal | ink, vet |
| 0 | soort weg |

## 03 Openklappen · A: Lade over de terminal

| | |
|---|---|
| Plek | boven de teller, rechts tegen de rand |
| Maat | 362 px breed, tot de titelbalk hoog |
| Terminal | blijft even groot, ligt eronder |
| Sluiten | klik op de teller of ernaast |

## 04 Indeling · A: Per soort

| | |
|---|---|
| Koppen | Poorten · Browsers · Artifacts |
| Per regel | de sessie in mute, rechts; wat niet past valt weg achter … |
| Zoals | het paneel nu |

