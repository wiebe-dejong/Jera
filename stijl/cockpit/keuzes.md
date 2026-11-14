# Stijl Cockpit: keuzes

Bord: https://claude.ai/artifact/6M7pWS1JLMd6jWtFqZkoj3 (bron: `keuzebord.html`)
Ronde 4, 2026-11-15. Vast 19, open 0, deels 0, later 0.

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
| Vorm | vierkantjes (hoek 0): commit 6 px, groep 10 px met het aantal ernaast, eindpunt 13 px hol; afsplitsen met een rechte hoek; lijn 2 px (2026-11-14, gekozen uit Fijn, Recht en Kaal: "C, maar dan recht van B") |
| Dagen | geen datumkolom: de dag rechts op de daglijn, in `klein` en `mute` (2026-11-14) |
| Opgehaald | rechts op de gestippelde lijn (2026-11-14, eerst in de datumkolom) |
| Inklappen | de knop git in de Render-regel; dicht krijgt de terminal de volle breedte |

Aangepast op verzoek: "maar inklapbaar". Verwerkt als: het hele paneel klapt in.

## Ronde 4: agents van een sessie

De subagents en teamleden van een sessie. De feiten staan in `research/agents.md`.

## 13 Agents, plek · A: Onder de sessie

| | |
|---|---|
| Plek | onder de sessieregel, 23 px ingesprongen, onder de titel |
| Rij | 26 px, zoals de sessieregel |
| Uitklappen · B: Pijltje | met het pijltje rechts in de sessieregel, dat van een map, gedempt. De gekozen sessie begint open |

## 14 Agents, dicht · C: Bezig en totaal

| | |
|---|---|
| Dichte sessie | bezig en totaal helemaal rechts in de sessieregel, gedempt, na het pijltje |
| Voorbeeld | `2/4` |

## 15 Agentregel · A: Naam

| | |
|---|---|
| Teamlid | zijn naam |
| Subagent | zijn omschrijving; een subagent heeft geen naam |

## 16 Kleur teamlid · C: Naam in zijn kleur

| | Donker | Licht |
|---|---|---|
| blue | `#6EA8FF` | `#1F5FD0` |
| green | `#5BC27E` | `#1F7A3D` |
| yellow | `#E8C547` | `#8A6A00` |
| purple | `#C08BE0` | `#8E44AD` |
| orange | `#F0975A` | `#B4531A` |
| pink | `#F28DC0` | `#B03A7A` |
| cyan | `#5CC8D6` | `#0E7A88` |
| red | `#FF7A7A` | `#C0392B` |

Een subagent heeft geen kleur en staat in de tekstkleur. Geel op licht haalt 4.7:1, paars op licht 5.4:1. Groen, geel en rood lijken op de statuskleuren. De regel in het design system dat kleur alleen status en git is, krijgt hiervoor een uitzondering.

## 17 Stand agent · A: Vinkje en streepje

| | |
|---|---|
| Bezig | groen draaiend boogje |
| Klaar | groen vinkje, naam gedempt |
| Gestopt | grijs streepje, naam gedempt |

## 18 Klaar en gestopt · A: Blijven staan

| | |
|---|---|
| Volgorde | zoals ze gestart zijn |
| Klaar en gestopt | blijven op hun plek |

## 19 Subagent en teamlid · B: Twee groepjes

| | |
|---|---|
| Kopjes | links `team` of `subagents`, rechts het pijltje en helemaal rechts het aantal, 11 px, gedempt |
| Uitklappen | elk groepje apart, met het pijltje |
| Volgorde | team eerst |

Aangepast op verzoek: "apart uitklapbaar". Verwerkt als: elk kopje krijgt het pijltje van een map en het aantal.

Aangepast op 15 november 2026, bij het bouwen: "ik wil de dropdown pijl rechts, en de 1,2,3 indicatie helemaal rechts". Geldt voor de kopjes en voor de dichte sessieregel.

Aangepast op 15 november 2026: "doe subagents maar alleen een getal, teammembers wel de dropdowns zoals nu". Subagents zijn één kopje met alleen het aantal, zonder pijltje en zonder lijst. Het team klapt open en dicht zoals hierboven.

Buiten het bord gebleven: een agent die op mij wacht (nog niet gemeten).

Gekozen op 15 november 2026, zonder bord: een klik op een teamlid kiest zijn sessie, net als een klik op de sessieregel.

## Onderdelen buiten het bord

Gekozen op 2026-11-14, zoals ze in het design system staan (https://claude.ai/artifact/QKTtDToW7LYRivNM1vYG88):

| Onderdeel | Kern |
|---|---|
| Knop | 20 px hoog, rand van 1 px. Gewoon in de tekstkleur, gevaar met rode tekst, de hoofdknop als vlak in de tekstkleur |
| Bestandsboom | rij van 26 px, 14 px inspringen per niveau, map met pijltje en plusteken, icoon links van de naam: GitLab voor bestanden, Catppuccin voor mappen (2026-11-14, eerst zonder icoon) |
| Eerder-regel | titel, dag gedempt, knop Hervat; zonder statusicoon |
| Poortenpaneel | groepen agent-browser en poorten met hun aantal; poortnummer vet vooraan; knoppen Openen en Stoppen |
| Dialoog | de vraag als kop, de betrokken sessies eronder, rechts onderaan de veilige knop en dan de actie |
| Lege staat | één regel gedempt en één knop; een fout gebruikt dezelfde vorm |

## Volgende stap

De keuzes worden tokens en code in de app zelf. Dat gebeurt in dit project, niet op het bord.
