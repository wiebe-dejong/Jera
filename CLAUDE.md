# Cockpit

Eigen Mac-app voor mijn ontwikkelwerk: Claude-terminals, servers, browser, documenten, git en deploys in één venster. Geen IDE.

## Status

Spec-fase. Schrijf geen app-code zolang SPEC.md de indeling, flows en het uiterlijk niet als **besloten** heeft staan. Indeling, flow en uiterlijk bespreek je eerst met mij. De techniek mag Claude zelf uitwerken.

## Bestanden

- `SPEC.md` beschrijft wat de app doet, met keuzes, hun status en open vragen. Werk het bij na elk besluit.
- `research/` bevat het onderzoek van 2026-11-04 met bronnen:
  - `werkwijze.md`: hoe ik nu werk, gemeten.
  - `bestaande-tools.md`: wat er al bestaat.
  - `zelf-bouwen.md`: stack, onderdelen, risico's.

## Werkafspraken

- Ontwerpprincipe: Claude voert uit, de cockpit laat zien en laat mij ingrijpen.
- De UI legt zichzelf uit, zie §5 van de globale CLAUDE.md. Toon status als icoon, kolom of badge, niet als zin.
- Houd de app licht. Hij draait op een M3 met 8 GB RAM, naast meerdere Claude-sessies.
- Zodra er een werkende build is: werk dagelijks in een stabiele build en ontwikkel in een aparte dev-build. Een herstart van de dev-build mag geen Claude-sessies of servers afsluiten.

## Stack (advies, nog niet gestart)

- Electron, electron-vite, React, TypeScript en Tailwind.
- xterm.js 6 met node-pty.
- WebContentsView met CDP.

Onderbouwing staat in `research/zelf-bouwen.md`.
