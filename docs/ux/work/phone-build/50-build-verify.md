---
work: phone-build
gate: 3
verdict: pass
date: 19 Nov 2026
---

# Gate 3: Build on a phone against the pen

## Setup

- Branch `feat/frontend-new-v2-phone-build`, with main merged at c0e749c9. That merge adds Sora's Design pane to the desktop.
- Phone viewport 390 × 844 with an iPhone user agent, dark and light.
- Run against the local backend with the fake model and no Shipyard. Pen frames read with `Get` (resolveInstances).

## Checked against the pen

- **Switch Chat and Preview (yS6ev):**
  - Nav: back, the project in mono, live, and ···.
  - Chat | Preview at full width, 44 high.
  - The preview runs full width with the Ask Bar ("Ask Koda for a change…", Koda's mascot) floating over it.
- **Project menu (wbexi):** the order is Deployments, Domain, Monitor, Database, Settings, then UNDER THE HOOD (Code, read-only; Terminal), then Share and Export.
  - The site variant says "No Database: this site doesn't store data." (§9).
  - Locally, Domain, the deploy number and Terminal are absent. They need Shipyard and the sandbox, the same as Publish on the desktop (§9). The real-backend e2e covers them with seeded publishes.
- **Deployments (pefkG):** where publishing isn't set up, it says so ("Publishing isn't set up here"), as on the desktop.
- **Code (C0kOpT):** "Code · read-only", ALL FILES, then "Read-only. Ask Koda to change anything."
  - RECENTLY CHANGED and the commit are left out (§9, BACKEND_TODO.md).
- **Publish failed (ALFZb):** the line above the card, "Build failed", "{STEP} SAID" with the message, the meta, Fix it (primary) and View log.
- **Desktop preview (PsC5F):** the device toggle, the desktop frame and the hint.

## Deviations found and fixed in this pass

1. **Desktop preview (PsC5F), high.**
   - The build filled the whole screen with the scaled page: tiny content, no frame, and the hint hidden under the Ask Bar.
   - The pen draws the 1440 layout in a 240-high card (`border-strong`, radius 10, padding 12), centred, with the hint in sans 13 `text-3` under it. It's now built that way.
2. **The device switch: redesigned in the pen, then built. Medium.**
   - The pen's centred Phone | Desktop pill (G0ih1) under Chat | Preview read as a second tab bar. In dark mode its chosen side sat darker than the track, so it looked recessed, not chosen. Mario flagged it ("this one is not good").
   - Decided under nomi-ux-rules, "best for mobile": the preview gets its own bar.
     - The pen now draws this bar as Preview Bar `GppXE` (dark) and `ScEam` (light) in PsC5F / n5JJS. The pill is hidden, not deleted, and the step caption is updated.
     - Layout: 40 high with a bottom stroke. The app's address in mono at 12, then Reload, Open in browser (left out where the address holds a secret, the sandbox's), then Phone | Desktop as two 28px icons on a surface-2 track, the chosen one raised (surface, border-strong).
     - Why: one tab bar instead of two, the choice reads in both themes, and Reload and Open give a way out when the app doesn't load.
   - Built as `projects/PhonePreviewBar`, with its docs entry, live example and component test.
3. **The preview while loading, medium (UX).** The frame was blank white until the app answered, which read as broken. It now says "Loading the preview…" until the frame has loaded.
4. **Failed card meta (ALFZb), high.**
   - It read "deploy #0 · publish step · 0:00" for a publish with no number and no finish time.
   - The pen's "deploy #2 · build step · 00:41" now shows only what's known: no "#0", no made-up "0:00", and minutes in two digits.
   - The log screen and the quote above the composer name the project when there's no number.
5. **Project menu's Export (wbexi), low.** "Export repo" is now the pen's "Export".

## Still open (suggestions, not blocking)

- **Ask Bar over the app's bottom:** on Phone it floats over the app's own foot, so a pre-order button at the bottom of the app sits under it. The pen draws it that way. Hiding it while the person scrolls the app would need the app's scroll position, which a sandboxed frame doesn't give.
- **A preview that can't be reached:** an app that never answers still ends on the browser's own blank error page inside the frame, which the app can't detect across origins. The bar's Reload and Open in browser are the way out.

## Not reached locally

- Publish, Publishing, roll back, and the live card: these need Shipyard.
- Terminal: needs the sandbox.
- Presence and follow: need a second person.

The real-backend e2e covers what can be seeded (`phone-build.e2e.ts`, `together.e2e.ts`, `asks.e2e.ts`).
