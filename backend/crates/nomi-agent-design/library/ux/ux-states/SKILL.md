---
name: ux-states
description: "Which states every screen and component must show (loading, three kinds of empty, error, edge), how forms validate and word errors, and how to lay the state frames out on the canvas and list them in the handoff. Read before drawing any list, form or login."
license: "Apache-2.0, Copyright 2026 Open Design contributors"
source: "https://github.com/nexu-io/open-design at commit 5fdcc3db1e31bd4fa70f673041b78e87977507c3 (craft/state-coverage.md, craft/form-validation.md, skills/login-flow/)"
---
# UX states: every screen in every state

The most common failure of a generated design is drawing only the full, happy screen. Every surface that shows, loads or takes data gets all five states below. `taste-app-screens` sets the look (layout at rest while loading, one line and one action when empty); this file sets which states exist and what each must hold.

## The five states

| State | When | Must hold |
|---|---|---|
| Populated | Data is there | The screen you designed |
| Loading | Data on its way | The layout at rest with Skeleton blocks where data goes; the shell (nav, page head) stays drawn |
| Empty | Nothing yet, or nothing matches | A heading, one plain line, one action (three kinds, below) |
| Error | Load, save or check failed | What happened, why if known, what to do; the person's input kept |
| Edge | Long, huge, missing or odd data | The same layout, unbroken (table below) |

Only the part that is fetching shows Loading: a slow order list never blanks the whole screen.

**Loading by expected wait** (note it for Koda): under 0.3 s nothing; 0.3–2 s Skeleton; 2–10 s Skeleton or a labelled Spinner ("Memuat pembayaran…"); at 15 s add one line "Masih memuat, sebentar lagi"; 10–60 s a Progress bar with "Batal"; past 60 s stop and show the Error state with "Coba lagi". Never a spinner that runs forever.

## Three kinds of empty

| Kind | Example | What it holds |
|---|---|---|
| First use | No orders yet | What will appear and when, plus the action that fills it: "Belum ada pesanan" / "Pesanan pertama muncul di sini begitu pelanggan membayar." / Button "Bagikan link toko" |
| No results | A search or filter found nothing | Echo the query, offer a way out: "Tidak ada hasil untuk 'lapis legit'" / "Coba kata lain atau hapus filter Juli 2026." / Button outline "Hapus filter" |
| Cleared | Everything done | A calm close and the optional next step: "Semua pesanan sudah dikirim." / link "Lihat riwayat" |

No illustration: Taste keeps empty states to words and one action; one outline icon from the set at most. An error is never drawn as an empty: "Belum ada pesanan" when the list failed to load is a lie.

## Errors

Every error answers three questions, in this order:
1. **What happened:** "Pembayaran ditolak bank." Never "Terjadi kesalahan."
2. **Why, if known:** "Saldo tidak cukup." / "Koneksi terputus."
3. **What to do:** a button ("Coba lagi", "Pakai QRIS"), another path, or a contact.

Match the error to its scope:

| Scope | How you draw it |
|---|---|
| Field | `$color-destructive` border, an icon and the message under the field |
| Form | An Alert / Destructive above the form naming each field ("2 isian perlu diperbaiki: Nomor WhatsApp, Alamat"), plus each field's own error |
| Section | An Alert inside that section with "Coba lagi"; the rest of the screen still works |
| Page | The page head stays; one heading, one line, the recovery action |
| App | One banner under the top bar ("Kamu sedang offline. Perubahan disimpan dan dikirim saat online.") |

Retry: the first retry is the person's tap. After three failures swap "Coba lagi" for "Hubungi kami" and show a short code they can copy ("Kode: PAY-2607"). After a retry show "Terakhir dicoba 12 detik lalu".

Toasts (Sonner) sit in one place in the whole app (bottom centre on a phone, bottom right on desktop) and carry their words, not colour alone.

## When a field speaks up

- Untouched: no error, no green tick.
- While typing for the first time: still nothing. "Why is it telling me my email is wrong, I haven't finished" is the most common complaint.
- On leaving the field: check it; show the error if it fails.
- Once it is in error: re-check as they type and clear the error the moment it is right.
- On submit: check everything, move to the first failed field (or to the Alert on a long form). Never clear what they typed. On the payment step never wipe the card or the address because another field failed.
- A slow check (is this shop name free?) never freezes typing or keeps the button disabled; the server's answer on submit is the final one.
- Accept input as people type it: "0812-3456-7890", "+62 812 3456 7890" and "081234567890" are the same number. Format it on display, never reject the dashes.
- Valid fields stay quiet: helper text stays, no success colour.

## Error copy (WCAG 3.3)

- Name the field and the rule that failed, never a catch-all. For busy fields write the specific messages: WhatsApp "Nomor terlalu pendek: minimal 10 angka", "Nomor harus diawali 08 atau +62"; email "Email perlu tanda @", "Domain email belum lengkap (contoh: nama@toko.example)".
- Suggest the fix when it can be known: "Tanggal lahir 31/02 tidak ada. Maksudnya 28/02/1996?"
- Money, legal or data that can't be undone: a review step before commit ("Periksa pesanan": items, total Rp 249.000, address, then "Bayar Rp 249.000"), or an undo, or a confirm Dialog for deletes.
- Don't ask twice: no "ulangi email", no retyping an address given on the step before; offer "Sama dengan alamat pengiriman".
- Login never tests memory or puzzles: allow paste in password and code fields, let password managers fill, accept a pasted OTP.

## Form fields as component variants

Draw the Field component (label, control, helper, error) with these states on the design system frame, named `Input / Default / <State>`: Default, Hover, Focus, Filled, Error, Disabled; plus Textarea, Select and Input OTP in the same set. Error = `$color-destructive` stroke, a 16px alert icon `path` and the message in `$color-destructive` under the helper. The form while sending = Button `Loading` ("Mengirim…") with the fields drawn as Filled; draw it once.

## Login and sign-up checklist

- Labels above fields, always visible; the placeholder is never the label and never a fake sample ("contoh@email.com"); put a format hint in the helper line.
- Password field with a show/hide eye icon in a 44 tap area.
- WhatsApp or phone login: a fixed "+62" prefix in the field, the number keypad noted for Koda, a 6-box Input OTP that accepts paste, "Kirim ulang dalam 0:45".
- Social sign-in with the real logo in full colour (Google), never an emoji or a drawn look-alike.
- Errors under their own field; primary button with Hover, Active and Loading states; every target 44 tall.
- "Lupa kata sandi?" and "Belum punya akun? Daftar" both present.

## Edge cases to draw or check

| Surface | Edge |
|---|---|
| List, card | A 200-character name ("Kue lapis legit pandan premium isi 20 potong untuk hantaran lamaran…"), no photo (palette tone plus initials), no secondary action |
| Table | 1.248 rows (pagination "1–20 dari 1.248"), all-number columns, sort and filter on at once |
| Form | Every optional field empty; every required field at its longest |
| Search | One letter, only symbols, more than 1.000 results |
| Detail | Every optional field missing (hide the row, never show "null" or a dash wall) |
| Money, dates | Rp 1.249.000 that wraps (the number never splits from "Rp"), 26 Jul 2026 in every date |
| Network | Offline and slow: the App banner and the Loading notes above |

## On the canvas

- Put each state as its own top-level frame in the same row, right of the populated screen, 120 apart: `Pesanan — Phone`, `Pesanan — Phone — Loading`, `— Empty first use`, `— Empty no results`, `— Error`, `— Edge`. Same `theme`, same element ids as the populated frame.
- Draw states at 390 first; add a desktop state frame only where its layout differs. Reuse refs so state frames stay cheap against the 300-element budget.

## In the handoff

List the states per screen so Koda wires each:

| Screen | States drawn | Trigger | Copy |
|---|---|---|---|
| Pesanan | Loading, Empty first use, Empty no results, Error, Edge | Load > 0.3 s; 0 orders; filter has no match; load fails | As on the frames |

Name any state you didn't draw and what Koda should do ("Cleared: same as first use, heading 'Semua beres'").

Adapted from open-design (nexu-io/open-design@5fdcc3d, Apache-2.0, © 2026 Open Design contributors): craft/state-coverage.md, craft/form-validation.md, skills/login-flow/SKILL.md, skills/login-flow/references/checklist.md. Changed: rewritten for the OpenPencil canvas.
