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
2. **Device toggle (G0ih1), medium.** The build used the generic `Segmented`: 44 high, no icons. The pen's toggle is 40 high (radius 10, inset 2), with smartphone and monitor icons at 14 and words at 13, the chosen side on bg at 500. It's now `projects/PhoneDeviceToggle`, with its docs entry, live example and component test.
3. **The preview while loading, medium (UX).** The frame was blank white until the app answered, which read as broken. It now says "Loading the preview…" until the frame has loaded.
4. **Failed card meta (ALFZb), high.**
   - It read "deploy #0 · publish step · 0:00" for a publish with no number and no finish time.
   - The pen's "deploy #2 · build step · 00:41" now shows only what's known: no "#0", no made-up "0:00", and minutes in two digits.
   - The log screen and the quote above the composer name the project when there's no number.
5. **Project menu's Export (wbexi), low.** "Export repo" is now the pen's "Export".

## Still open (suggestions, not blocking)

- **Ask Bar over the app's bottom:** on Phone it floats over the app's own foot, so a pre-order button at the bottom of the app sits under it. The pen draws it that way. Hiding it while the person scrolls the app would need the app's scroll position, which a sandboxed frame doesn't give.
- **A preview that can't be reached:** an app that never answers still ends on the browser's own blank error page inside the frame, which the app can't detect across origins. An "Open in browser" link beside the hint would give a way out.

## Not reached locally

- Publish, Publishing, roll back, and the live card: these need Shipyard.
- Terminal: needs the sandbox.
- Presence and follow: need a second person.

The real-backend e2e covers what can be seeded (`phone-build.e2e.ts`, `together.e2e.ts`, `asks.e2e.ts`).
