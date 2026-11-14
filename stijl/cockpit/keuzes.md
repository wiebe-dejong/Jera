# Stijl Cockpit: keuzes

Bord: https://claude.ai/artifact/6M7pWS1JLMd6jWtFqZkoj3 (bron: `keuzebord.html`)
Ronde 2, 2026-11-14. Vast 12, open 0, deels 0, later 0.

Vooraf: vier hele richtingen op https://claude.ai/artifact/Sygn2xBSb5ugrKXaNy1uJF. Daaruit gekozen: de grond en de tab van richting B.

## Stand na ronde 2

## 01 Grond · A: Bijna zwart, één vlak

| | Donker | Licht |
|---|---|---|
| Grond | `#0E1013` | `#F6F6F2` |
| Balk | `#14171B` | `#ECECE6` |
| Lijn | `#23272E` | `#D9D9D0` |
| Lijn, sterker | `#333944` | `#C2C2B8` |
| Tekst | `#D5D9E0` | `#1E2227` |
| Gedempt | `#8A93A2` | `#5A616B` |
| Gekozen rij | `#1C2129` | `#E2E2DA` |

Panelen zijn één vlak met een lijn van 1 px ertussen. De terminal heeft dezelfde grond als de app.

## 02 Tab · A: Vlak met streep erboven

| | |
|---|---|
| Plek | in de titelbalk, 34 px hoog |
| Actief | grond van het venster, vet, streep van 2 px in accent |
| Niet actief | gedempt, lijn van 1 px ertussen |
| Teller | aantal sessies, gedempt |

## 03 Letter van de app · A: Mono overal

| | |
|---|---|
| Titels en knoppen | mono 12.5 px |
| Tabs, mappen, tellers | mono 12.5 px |
| Terminal en branchnamen | mono |

## 04 Mono-letter · A: JetBrains Mono

| | |
|---|---|
| Maat | 12.5 px |
| Regelhoogte | 1.55 |
| Herkomst | meegeleverd in de app |

## 05 Accent · A: Groen

| | Donker | Licht |
|---|---|---|
| Accent | `#5FD7A7` | `#0A6B4B` |

Gebruik: de streep op de actieve tab, op de schakelaar, op de knop git en op de gekozen sessie.

## 06 Vorm · A: Recht

| | |
|---|---|
| Gekozen rij | hoek 0, van rand tot rand |
| Invoerveld | hoek 0 |

## 07 Dichtheid · B: Rij van 26 px

| | |
|---|---|
| Rij | 26 px |
| Marge links | 12 px |

## 08 Statussen · B: Icoon per status

| Status | Icoon | Donker | Licht |
|---|---|---|---|
| Bezig | groen draaiend boogje | `#4FCB7C` | `#1A7F3C` |
| Wacht op antwoord | geel rondje met vraagteken | `#F2CC60` | `#AD8200` |
| Wacht op toestemming | rood slot | `#FF6B6B` | `#C9302C` |
| Net klaar | groen vinkje | `#4FCB7C` | `#1A7F3C` |
| Idle | grijs streepje | `#8A93A2` | `#5A616B` |

Maat 14 px. Geen woord in de regel; het woord verschijnt als je het icoon aanwijst.

## 09 Sessieregel · C: Streep in accent

| | |
|---|---|
| Icoon | voor de titel |
| Gekozen | vlak, vet, streep van 2 px links in accent |
| Idle | titel gedempt |

## 10 Schakelaar · B: Kleine letters met streep

| | |
|---|---|
| Tekst | gewone maat, kleine letters |
| Actief | vet, streep van 2 px in accent |
| Hoogte | 30 px |

## 11 Render-regel · C: Icoon en naam

| | |
|---|---|
| Status | groen vinkje (live) of rood kruis (deploy mislukt) |
| Link | een klik op de naam opent de app |

## 12 Git-paneel · B: Getrapte namen

| | |
|---|---|
| Namen | recht, boven de lijnen, elke naam op een eigen regel; de hoofdbranch bovenaan |
| Kop | 88 px hoog bij vier branches (18 px per naam) |
| Paneel | 346 px breed bij webshop |
| Opgehaald | in de datumkolom |
| Inklappen | de knop git in de Render-regel; dicht krijgt de terminal de volle breedte |

Aangepast op verzoek: "maar inklapbaar". Verwerkt als: het hele paneel klapt in.

## Onderdelen buiten het bord

Gekozen op 2026-11-14, zoals ze in het design system staan (https://claude.ai/artifact/QKTtDToW7LYRivNM1vYG88):

| Onderdeel | Kern |
|---|---|
| Knop | 20 px hoog, rand van 1 px. Gewoon in de tekstkleur, gevaar met rode tekst, de hoofdknop als vlak in de tekstkleur |
| Bestandsboom | rij van 26 px, 14 px inspringen per niveau, map met pijltje en plusteken, bestand zonder icoon |
| Eerder-regel | titel, dag gedempt, knop Hervat; zonder statusicoon |
| Poortenpaneel | groepen agent-browser en poorten met hun aantal; poortnummer vet vooraan; knoppen Openen en Stoppen |
| Dialoog | de vraag als kop, de betrokken sessies eronder, rechts onderaan de veilige knop en dan de actie |
| Lege staat | één regel gedempt en één knop; een fout gebruikt dezelfde vorm |

## Volgende stap

De keuzes worden tokens en code in de app zelf. Dat gebeurt in dit project, niet op het bord.
