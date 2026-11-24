---
name: practice-login-flow
description: "How to design a phone-first sign-in flow for an Indonesian app: choosing the method (phone and OTP, WhatsApp OTP, Google, password), the screen sequence, the 390 layout, keyboards and safe areas, and which states to draw as frames. Read when an app needs masuk or daftar screens."
license: "Apache-2.0, Copyright nexu-io (open-design); via open-design (Apache-2.0)"
source: "https://github.com/nexu-io/open-design/tree/5fdcc3db1e31bd4fa70f673041b78e87977507c3/skills/login-flow; open-design@5fdcc3d"
---
# Sign-in flow on a phone

Design the masuk and daftar screens of an app at 390 wide: pick the method first, then draw each step and its states as frames.

The acceptance checklist for these screens (labels above fields, show/hide password, error text under the field, 44 targets, loading in the button, forgot-password and sign-up links) lives in `ux-states`; run it there. This skill covers the decisions around it.

## 1. Choose the method first

Ask, or decide from the brief, before drawing:

| Method | Fits | Notes |
|---|---|---|
| Phone number + OTP by SMS | most Indonesian consumer apps; customers who never use email | the default; no password to forget |
| Phone + OTP by WhatsApp | when the business already lives on WhatsApp | offer SMS as the fallback link |
| Google sign-in | younger users, office tools, sellers with Gmail | one button, plus another method for those without |
| Email + password | B2B tools, admins, staff accounts | add "Lupa kata sandi?" |
| Email magic link | low-frequency tools | needs a "cek email Anda" screen |

Most apps need one main method and at most one alternative. Show at most two social buttons (Google, and Apple only if the brief targets iPhone users). Social buttons carry the real full-colour logos (Taste rule 7), never emoji or a drawn look-alike.

## 2. The screen sequence (phone + OTP)

1. **Masuk:** logo, a heading that says what happens ("Masuk ke Warung Bu Sri"), the phone field with a country prefix chip (+62 default), the main button "Kirim kode", then "atau" and the alternative method, then "Belum punya akun? Daftar".
2. **Kode OTP:** heading "Masukkan kode", the line "Kode 6 digit dikirim ke 0812-xxxx-x123" with "Ubah nomor", six digit boxes (`Input OTP`), a resend link with a countdown ("Kirim ulang dalam 0:45"), and "Kirim lewat SMS" if WhatsApp was first.
3. **Data diri (first time only):** name, and only what the app truly needs now. Everything else waits until it's needed.
4. **Done:** straight into the app's home; no welcome tour unless the brief asks.

For email and password, the sequence is Masuk, then Lupa kata sandi (email field, "Kirim tautan"), then a "Cek email Anda" screen with the address shown and "Kirim ulang".

## 3. Layout at 390

- Draw at 390 wide (the open-design source uses 375; Nomi uses 390), with the status bar area (44-47 top) and the home-indicator area (34 bottom) kept clear: put nothing tappable there.
- One column, 20-24 side padding (`$space-5`/`$space-6`), content starting about a third down or top-aligned below the logo; never vertically centred in a way the keyboard will cover.
- The main button full width, at the bottom of the form, not pinned to the screen bottom where the keyboard hides it.
- No sideways scroll; labels and errors wrap.
- Show one frame with the keyboard up (a grey block about 300 high at the bottom) to prove the button and the active field stay visible.

## 4. Field details to annotate for Koda

| Field | Keyboard | Autofill note |
|---|---|---|
| Phone | numeric phone pad | phone number; format as typed: 0812-3456-7890 |
| OTP | numeric | one-time code (the phone offers the SMS code) |
| Email | email keyboard | email |
| Password | default, hidden text | current password, or new password on sign-up |
| Name | default, capitalised words | name |

## 5. States to draw as frames or components

- Default, field focused (`$color-ring`), filled.
- Error under the field: "Nomor belum lengkap" or "Kode salah, coba lagi" (say what to do, not just "invalid").
- Loading: spinner inside the main button, button disabled.
- OTP expired: resend link active, the countdown gone.
- Too many attempts: a calm note with the wait time ("Coba lagi dalam 5 menit").
- No connection: toast "Tidak ada koneksi. Coba lagi."

Name them `Masuk — Phone / Error`, `OTP — Phone / Expired` and keep the same element ids across them, so feedback lands on the right element.

## Copy

- Heading names the place: "Masuk ke <nama app>", not "Welcome back".
- The button says what happens next: "Kirim kode", "Masuk", "Daftar".
- Never a sample address like "contoh@email.com" without saying it's an example; prefer the label alone.

Adapted from open-design's login-flow skill (Apache-2.0, nexu-io/open-design), via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
