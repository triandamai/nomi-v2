---
name: practice-research-decision-room
description: "How to turn messy evidence (customer chats, Google reviews, the owner's notes, feedback on your versions) into an honest design decision: an evidence ledger, themes with confidence, scored options and a one-minute memo, drawn as a decision frame. Read when the person asks which direction is right."
license: "Apache-2.0, Copyright nexu-io (open-design); via open-design (Apache-2.0)"
source: "https://github.com/nexu-io/open-design/tree/5fdcc3db1e31bd4fa70f673041b78e87977507c3/skills/research-decision-room; open-design@5fdcc3d"
---
# Research decision room

Turn mixed signals into one clear, honest next move, without making weak evidence look stronger than it is.

Use it when the person brings evidence and a question: "Pelanggan bilang susah pesan, apa yang harus diubah?", "Lebih baik katalog atau langsung WhatsApp?", "Kenapa orang berhenti di halaman harga?". Don't use it for picking a look or a mood: that is a direction choice (`design-loop`).

## Step 1: frame the decision

Write five lines before touching evidence:

- **Question:** "Apakah halaman pesan dibuat satu langkah atau tiga langkah?"
- **Who:** the buyer or user group ("pelanggan baru dari Instagram").
- **When:** the horizon (this version, this month).
- **Constraints:** budget, what Koda can build, what the owner won't do.
- **Out of scope:** what this decision will not settle.

If the person gave no question, derive one from the evidence and label it "disimpulkan". Missing context that doesn't block you becomes a labelled assumption, not a broad question back.

## Step 2: the evidence ledger

One row per useful signal:

| Field | Example |
|---|---|
| id | C-03 (chat), R-07 (review), F-02 (feedback on a version), M-01 (a metric) |
| source type | chat, review, observed use, survey, metric, owner's note, stakeholder |
| who | "pelanggan baru", "reseller", or "tidak diketahui" |
| signal | one sentence: "Bingung ongkir baru muncul di akhir" |
| quote or number | the person's own words or figure, or "tidak ada" |
| strength | strong, medium, weak |
| limits | why it may be biased: one loud customer, old review, secondhand |

**Strength.** Strong only when the same pain comes from several independent sources, was seen happening (not just said), or a number and a complaint point the same way. Medium: specific and plausible but from one source type. Weak: anecdotal, secondhand, undated, or mostly the owner's opinion.

**The owner's or a stakeholder's opinion is context, not user evidence.** Mark it separately; never count it as a customer signal.

**Never invent** quotes, counts, dates, names or revenue. A missing number is "tidak ada", plus what evidence would raise confidence.

## Step 3: themes

Cluster into 4 to 6 themes. Each:

- A plain name with a verb, describing behaviour: "Pembeli baru menyerah saat diminta daftar akun dulu", not "Masalah onboarding".
- The ids that support it (at least two, or it is labelled low confidence).
- The behaviour behind it, not just the screen complaint.
- Confidence: **high** (repeated, behaviour-backed, tied to the question), **medium** (plausible, missing depth), **low** (worth exploring, not worth building on).
- Contradictions and differences between groups, shown, never hidden.

Confidence is not an average of strengths: ask whether it explains behaviour, repeats across sources, has a plausible other explanation, and could be tested quickly.

## Step 4: score 3 to 5 options

| Score 1-5 | 1 | 3 | 5 |
|---|---|---|---|
| Evidence | anecdotal | repeated in one source type | repeated across sources, or behaviour plus number |
| Pain | mild friction | slows the task | blocks ordering, trust or return visits |
| Leverage | unclear | helps a known step | sits on the main action (the order, the booking) |
| Risk (5 = low) | large, hard to undo | contained, several screens | small, reversible, easy to test |

Show the total, then one sentence on why the winner wins. The score starts the conversation; it never replaces judgement.

## Step 5: the one-minute memo

1. Recommended move, labelled **direkomendasikan**, **sementara** or **perlu bukti lagi**.
2. Why now.
3. Which evidence ids support it.
4. What could be wrong.
5. What to watch after launch (a number and the threshold that counts as success).
6. The reversible first step.

If the recommendation rests on weak evidence, the first step is a small test or a prototype version, never a full redesign.

## Step 6: draw it (when the person wants to see it)

A single desktop frame (1440 wide, `fit_content` height) named `Keputusan — <topik>`, sections in this fixed order: question with confidence and date; the recommendation, risk and next test; the ledger as a table; themes with their ids; the option matrix; the memo; a test queue (what, who, metric, threshold); assumptions and limits. A 390 version stacks the same sections with no sideways scroll.

Encode confidence and risk with a text label first and colour second, sparingly (`$color-muted` grounds, the destructive and a success token only for risk and confidence chips). No violet or indigo accents.

## Check before you share

- The question is visible at the top; the recommendation is labelled.
- Every theme cites two ids or says low confidence.
- Quotes only from the person's material; missing figures say "tidak ada".
- Opinions are marked apart from customer evidence.
- All four score dimensions shown; the memo has "what could be wrong" and "what to watch".
- No lorem ipsum, invented customers or logos.

Adapted from open-design's research-decision-room skill (Apache-2.0, nexu-io/open-design), via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
