---
name: practice-faq-page
description: "How to design an FAQ section or help page: when it needs search and categories, real questions with short answers from the brief's facts only, and the accordion's open, closed, focus and no-result states. Read when a brief asks for FAQ or bantuan."
license: "Apache-2.0, Copyright nexu-io (open-design); via open-design (Apache-2.0)"
source: "https://github.com/nexu-io/open-design/tree/5fdcc3db1e31bd4fa70f673041b78e87977507c3/skills/faq-page; open-design@5fdcc3d"
---
# FAQ section and help page

Design questions people really ask, answered in a few sentences, in an accordion that scales from a short list to a searchable help page.

## Size decides the shape

| Questions | Shape |
|---|---|
| 1-5 | a plain list or an accordion in one section; no search, no categories |
| 6-11 | one accordion section on the landing page, maybe two groups under small headings |
| 12-18 | its own page: header, search field, 3-4 category chips, grouped accordion, a contact close |
| more | a help centre: categories as cards leading to their own lists; out of scope for one page |

On a landing page the FAQ is one of the few sections allowed without a picture (Taste), and it never sits next to another picture-less section (price list, specs, booking form).

## The content comes first

- **Questions from the brief and the trade, answers from the brief's facts.** You may draft the questions buyers of this trade ask (ongkir, jam buka, cara bayar, reschedule, garansi). Answer only with facts the brief gives. A question you can't answer truthfully is left out or listed in your handoff as "perlu jawaban dari pemilik"; never invent a policy, price, guarantee or refund rule (`taste-copy`).
- **Write the question as the buyer says it,** in first person, short: "Bisa kirim ke luar Jawa?", "Kalau batal, uang kembali?", "Perlu booking dulu?".
- **Answer in 2 to 4 sentences,** the yes or no first, then the condition, then the next step: "Bisa. Kami kirim lewat JNE dan SiCepat ke seluruh Indonesia. Ongkir dihitung otomatis saat checkout."
- An answer over about 100 words is split into short paragraphs or a list of steps.
- No question that only exists to sell ("Kenapa harus pilih kami?").
- The section heading names the thing: "Pertanyaan", "Pertanyaan soal pengiriman". FAQ entries are the one place a question may be a heading.

Typical groups for a small business: **Pemesanan**, **Pembayaran**, **Pengiriman** or **Kedatangan**, **Akun** (only if there is one), **Umum**.

## Drawing it

Page order for a full FAQ page: header (title plus one line on what can be found), search field, category chips with "Semua" selected, the grouped accordion, then a close: "Masih ada pertanyaan?" with the main contact action (a form, or "Chat lewat WhatsApp" only when the brief gives the number).

Components (shadcn names, `components-kit`):

| Part | Component | States to draw |
|---|---|---|
| Question row | `Accordion / Item / Closed`, `Accordion / Item / Open` | closed, open, hover (accent wash), keyboard focus (`$color-ring`) |
| Expand mark | a chevron or plus `path` icon, one style per page | rotated in the open state |
| Search | `Input / Search / Default` with a leading search icon | default, focused, filled |
| Category chips | `Toggle Group / Chip / Selected` and `/ Default` | selected, default, hover |
| No result | `Empty / Search / Default` | "Tidak ada pertanyaan yang cocok dengan 'refund'." plus a contact link |

- The whole row is the tap target, at least 44 high on 390, the question text wrapping (`fixed-width`).
- Show one item open in the main frame so the person sees an answer; note for Koda whether opening one closes the others (default: several may stay open).
- Separate items with a `$color-border` hairline, not cards: two boxes deep at most.
- Note for Koda in a canvas note: search matches question and answer text, ignores case; chips filter by group; expand and collapse animate height briefly.

## Check

- Questions are specific to this business, answers are 2-4 sentences, every fact traceable to the brief.
- Open, closed, focus and no-result states exist as frames or components.
- Works at 390 with a usable search field and tappable rows.
- One contact action at the end, worded exactly like the page's other contact buttons.

Adapted from open-design's faq-page skill (Apache-2.0, nexu-io/open-design), via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
