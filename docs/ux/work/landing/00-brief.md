---
work: landing
status: partly built
design: Figma, Nomi Builder (HFetdwcPvErbbtDKK87j3I) → 05 Marketing → "Landing v2 · Playful (ID)" 170:2766
---

# Landing page

The public page for people who aren't signed in, at `/welcome` in frontend-new.

| Section | Node | State |
|---|---|---|
| Nav | 170:2767 | built (Waitlist and Tanya-tanya links wait for their sections) |
| Hero | 170:2777 | built, with its motion (170:2766 timeline) |
| How it works | 170:2850 | built |
| Crew | 170:2895 | built |
| Own everything | 170:2915 | built, two lines changed (below) |
| Waitlist CTA | 170:2948 | left out: no public waitlist in the backend (BACKEND_TODO.md) |
| FAQ | 170:3016 | not read yet |
| Final CTA | 170:3053 | not read yet (its motion is known: the mascot parade hops) |
| Footer | 170:3071 | not read yet |
| Dark | 170:3097 | not read yet; the page is light until then |
| Mobile 390 | 170:3425, dark 170:3760 | not read yet; narrower screens reflow the desktop layout |

"Not read yet": the Figma MCP's View-seat call limit was reached partway through. Reading the rest
needs a Dev or Full seat, or the limit to reset.

## Decisions
- **Where:** `/welcome`, outside the app's shell; signed-in people go to Home. `/` still sends a
  signed-out visitor to sign in (pointing it at `/welcome` is a separate, one-line choice).
- **The composer** posts to the page's action: signed out, it goes to sign in with
  `redirect_to=/chat/new?q=…`; `/chat/new` puts the words in the field, unsent. The chips fill the
  field. Attach and the mic go to sign in too (they need an account).
- **Honesty:** "Kode aplikasi lengkap dengan riwayat perubahannya" → "…beserta tabel datanya" (an export
  has no change history; it has the code and the database's tables) and the folder card's "kode +
  riwayat" → "kode + data". "Bisa jalan offline dengan Ollama atau LM Studio" is left out: the
  backend's catalog has Ollama Cloud only, nothing on your own machine.
- **System:** the page keeps the marketing design's own values (`[data-landing]` in
  `src/lib/landing/landing.css`), not the app's tokens; Geist stands in for Inter.
- **Motion:** the design loops its whole 6s timeline, entrance included (a prototype's loop); the page
  plays the entrance once and loops the hops, the notes' bob and Maya's breathing. `prefers-reduced-motion`
  stops all of it.

## Checked
- 1440: the five sections measure 4,079px tall in Indonesian against the design's 4,083.
- 390: no sideways scroll, every section reflows (interim, until the mobile frame).
