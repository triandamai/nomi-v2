---
work: landing
status: built (waitlist, dark and 390 pending)
design: Figma, Nomi Builder (HFetdwcPvErbbtDKK87j3I) → 05 Marketing → "Landing v2 · Playful (ID)" 170:2766
---

# Landing page

The public page for people who aren't signed in, at `/welcome` in frontend-new.

| Section | Node | State |
|---|---|---|
| Nav | 170:2767 | built (the Waitlist link waits for its section) |
| Hero | 170:2777 | built, with its motion (170:2766 timeline) |
| How it works | 170:2850 | built |
| Crew | 170:2895 | built |
| Own everything | 170:2915 | built, two lines changed (below) |
| Waitlist CTA | 170:2948 | left out: no public waitlist in the backend (BACKEND_TODO.md) |
| FAQ | 170:3016 | built from its layer metadata and the render; answers 2–6 written (below) |
| Final CTA | 170:3053 | built from its layer metadata, the render and its motion (the parade hops) |
| Footer | 170:3071 | built from the render; links only what has a page |
| Dark | 170:3097 | not read yet; the page is light until then |
| Mobile 390 | 170:3425, dark 170:3760 | not read yet; narrower screens reflow the desktop layout |

The Figma MCP's View-seat call limit was reached partway through. The FAQ, final CTA and footer were
built from what had been read (their layers' metadata: positions, sizes, text; the motion data; the
full-page render, with colours measured off it: the CTA wash #f6fce2 → #ecfaf8). Check them against
their design context once it can be read. The dark and 390 frames weren't read at all.

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

- **FAQ answers.** The design writes only the first. The others say what Nomi does: a chat vs a
  project (Koda's workspace), the models (Nomi's own or a connected provider; the composer names
  it), your own key yes and a laptop AI not yet (cloud providers only), where data is and who sees it
  (yours unless shared; a shared workspace's members see its projects; memory in You › Memory), and
  teams (workspaces, invites by email, roles, building together).
- **Footer.** Only links with a page: Cara kerja, Kru, Tanya-tanya. Harga, Template, Panduan, Status
  and the Nomi Labs column (Tentang kami, Privasi, Ketentuan) wait for their pages. The languages
  switch the page (Paraglide's cookie).
- **Composer edge.** Empty, the quiet border (as the closing call draws it); with words in it, the
  strong one (as the hero draws it).

## Checked
- 1440: nav to Own everything measure 4,079px tall in Indonesian against the design's 4,083; the whole
  page is 5,737 against 5,806 without the waitlist, the difference the footer's links left out.
- 390: no sideways scroll, every section reflows (interim, until the mobile frame).
