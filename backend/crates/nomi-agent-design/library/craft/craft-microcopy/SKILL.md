---
name: craft-microcopy
description: "Vercel's Writing Guidelines applied to the words inside an interface: voice, banned and weasel words, AI-generated tells, headings, lists, units, placeholders and punctuation, with Indonesian examples. Read it when writing labels, errors, empty states, help text and in-app explanations."
license: "MIT, Copyright (c) 2026 Vercel Labs (writing-guidelines), via open-design (Apache-2.0)"
source: "https://github.com/vercel-labs/writing-guidelines @83e2316 (pinned in open-design@5fdcc3d skills/writing-guidelines)"
---
# Microcopy: the words inside an interface

How you write the small words on screens (labels, buttons, errors, empty states, help text, settings, onboarding) so they are plain, specific and never sound generated.

`taste-copy` already governs a page's copy: its five rules, word budgets, slop words and devices. This skill adds the rules for interface text and for the longer explanations an app carries (help pages, settings descriptions, onboarding steps). Where they overlap, `taste-copy` wins.

## Voice

- **Active voice.** The "oleh monyet" test, Indonesian version of the source's "by monkeys": if you can add "oleh ..." to the sentence and it still parses, it is passive; rewrite. "Pesanan dibatalkan" becomes "Kamu membatalkan pesanan" only when the person did it; for a system action, name the system: "Toko membatalkan pesanan karena stok habis".
- **Talk to the person:** "kamu" or "Anda", one of them for the whole product (decided with the brand voice). Never "pengguna" about the reader.
- **Imperative for steps:** "Ketuk **Tambah produk**", not "Anda perlu mengetuk tombol Tambah produk".
- **Short sentences:** aim under 20 words. One idea per sentence.
- **Present tense** unless something really happens later.
- **No rhetorical questions:** "Bingung mulai dari mana?" reads as marketing; say what to do.
- **Second-read test:** read each line once at speaking speed. If you have to read it again, name the subject, the action and the result.

## Words to cut or replace

- **Pressure words:** "mudah", "gampang", "cepat", "simpel", "tinggal" about the reader's task. Say the concrete thing: "satu langkah", "pakai pengaturan bawaan".
- **Filler:** "sangat", "benar-benar", "hanya", "cukup", "sebenarnya".
- **Weasel words:** "banyak", "sering", "biasanya", "secara signifikan". Give the number or drop the claim: "Pesanan masuk rata-rata dalam 2 menit" only if the brief gives it.
- **Metaphor verbs** in place of the literal step: "mengalir ke", "mendarat di", "membawa". Say "terkirim ke", "muncul di".

## AI-generated tells (lint for these)

- **Recap openers:** a paragraph that starts by summarising the last one ("Setelah semua siap…", "Sekarang kita sudah…"). Start with the next point.
- **Stop-start fragments:** "Dulu manual. Sekarang otomatis. Hemat waktu." Join one dependent idea into one sentence.
- **Spec-sheet voice:** "menyediakan", "dapat dikonfigurasi", "tersedia fitur". Say what the person can do: "Kamu bisa mengatur jam buka per hari".
- **Personified objects:** "aplikasi memegang data", "sistem menyerahkan link". Use plain mechanics: "data tersimpan di", "link terkirim ke".
- **Template framing:** "Pertanyaan yang sering dihadapi pemilik usaha adalah…". The angle comes from this screen.

## Headings and labels

- Sentence case everywhere ("Atur jam buka"), which matches Taste and Indonesian usage.
- A heading says what the section holds; the reader can guess the content from it. Not "Info", "Lainnya", "Catatan": "Batas ongkir gratis".
- Nav labels short and scannable, verbs where they are actions: "Pesanan", "Produk", "Atur toko".
- Button: verb plus object. "Simpan produk", "Kirim tagihan", "Tambah varian". `&` only where space is truly tight.

## Structure of longer text

- Open every help page or long settings section with one sentence saying what it covers.
- Paragraphs of two to four sentences; one idea each.
- Spell out an abbreviation the first time: "Nomor Pokok Wajib Pajak (NPWP)". Explain every term the first time.
- Three or more items in a sentence become a list. Introduce a list with a colon. Numbered only for real order (steps).
- Bold only for a UI element name or one critical fact, never for tone.

## Numbers, units and placeholders

- A space between number and unit: "64 KB", "200 ms", "5 kg"; seconds bare in tight UI: "30s" (or "30 detik" in running text).
- Money and dates in Indonesian formats (`craft-interface-guidelines`).
- Example values in help text look obviously fake and copyable: `nama_toko_anda`, `081234567890`; never `<TOKEN>`, `xxx`, `ABC123`. On a public page, Taste's rule still holds: no invented phone number printed as real.

## Punctuation

- No em dashes as punctuation: use a colon, a comma or a full stop.
- Curly quotes, the single-character ellipsis, loading states ending with it: "Memuat…", "Menyimpan…".
- No exclamation marks, even in success: "Produk tersimpan." not "Berhasil!".
- Non-breaking space between a number and its unit, and inside brand names.

## Tone by situation

| Situation | Tone | Example |
|---|---|---|
| Onboarding step | Warm, predictable, no traps | "Pertama, tambahkan satu produk. Foto bisa menyusul." |
| A task in progress | Terse, direct | "Pilih kurir" |
| Reference (settings, limits) | Neutral, complete | "Maksimal 10 foto per produk, masing-masing 5 MB." |
| Explaining a concept | Like the reader will explain it back | "Stok minimum: jumlah saat kamu mau diingatkan untuk belanja lagi." |
| Something went wrong | Acknowledge, then fix; no apology theatre, no "Oops" | "Pembayaran belum masuk. Cek lagi dalam 10 menit atau hubungi bank." |

## Prices and plans

Pricing gets more detail, not less: a table, what is included, what counts as one unit ("1 pesanan = 1 transaksi, berapa pun jumlah barangnya"), limits, and what happens when they are passed. Never assume the person knows the model.

## Quick check

- Every label and button is specific; no "OK", "Lanjut", "Submit" where a verb and object fit.
- Every error says what happened and what to do.
- No pressure, filler or weasel words; no AI tells from the list above.
- Units spaced, formats Indonesian, ellipses and quotes typographic, no em dashes, no exclamation marks.

Adapted from Vercel's Writing Guidelines (MIT, Copyright (c) 2026 Vercel Labs), via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
