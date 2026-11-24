---
name: practice-paywall-upgrade
description: "How to design in-app upgrade moments that respect the user: when a paywall may appear, the parts of an upgrade screen, lock, limit and trial-ending screens, mobile paywalls and dark patterns to avoid. Read when an app has a free tier, trial or paid plans."
license: "MIT, Copyright (c) 2025 Corey Haines (mobile patterns from Blake Anderson's ai-app-playbook, MIT); via open-design (Apache-2.0)"
source: "https://github.com/coreyhaines31/marketingskills@5b2b400 (skills/paywalls, formerly paywall-upgrade-cro, renamed in #291; Nov 2026); open-design@5fdcc3d"
---
# Practice: paywalls and upgrade screens

How to design in-app upgrade moments that respect the user: when a paywall may appear, the seven parts of an upgrade screen, the feature-lock, usage-limit and trial-ending screens drawn as canvas frames, mobile subscription paywalls, and the dark patterns you never draw.

This is for apps (a cashier app with a free tier, a booking tool with a trial, a course app). A public pricing section on a website is `pattern-landing-pricing`; app screen rules are `taste-app-screens`.

## Ask first

1. **Upgrade kind:** free to paid, trial to paid, a higher tier, a paid add-on, a usage limit?
2. **The model:** what is free, what is paid, what triggers the prompt today?
3. **The journey:** when does it appear, what has the user already done, what are they trying to do at that moment?
4. **The "aha" moment:** the first time the user gets real value (first nota sent, first booking received). The ask comes after it.

## Four principles

1. **Value before the ask.** The user has already got something real; the upgrade is the natural next step.
2. **Show, don't just tell.** A preview of the paid feature with the user's own data beats a feature list.
3. **A short path.** Easy to upgrade when ready; the price is never hidden.
4. **Respect the no.** Easy to carry on free, so trust survives for a later yes.

## When a paywall appears

| Trigger | What the screen must do |
|---|---|
| **Feature gate:** user taps a paid feature | Say why it's paid, show what it does, a quick way to unlock, a way back |
| **Usage limit:** user hits the cap | Say the limit plainly, show what upgrading gives, don't block abruptly; offer a free way out (delete an old item, wait for next month) |
| **Trial ending** | Warn at 7, 3 and 1 day; say exactly what happens on expiry; summarise what they did in the trial |
| **Time-based prompt** after some days of use | Gentle, easy to dismiss, names unused paid features |

**Never show it:** during onboarding before any value, in the middle of a task (mid-checkout, mid-form), or again right after a dismiss. Annotate frequency for Koda: a limit per session and a cool-down of days, not hours, after "Nanti saja".

## The seven parts of an upgrade screen

| # | Part | Example (cashier app) |
|---|---|---|
| 1 | Headline: what they get | "Buka laporan stok harian" |
| 2 | Value shown | The stock report preview drawn with their own products and numbers |
| 3 | Comparison | Free vs Pro rows that really differ, their current plan marked "Paket Anda" |
| 4 | Price | Clear amount, unit and period: "Rp 99.000 / bulan" or "Rp 990.000 / tahun (2 bulan gratis)" |
| 5 | Proof | Only real proof from the brief; otherwise leave it out (Taste wins) |
| 6 | Main button: verb plus value | "Buka laporan stok", "Mulai Pro", "Lanjut dengan Pro" |
| 7 | Escape hatch | "Nanti saja" or "Tetap pakai gratis", visible, same screen, never tiny grey text |

## The three screens, as canvas frames

Draw each as a reusable component named the shadcn way (`components-kit`): a Dialog on desktop, a Sheet or Drawer from the bottom on the phone (390 wide), or a full screen in a mobile app.

**`Paywall / Feature Lock / Default`**
- Lock icon (path, 20-24) beside a short label "Fitur Pro".
- Preview of the feature (UI drawn with real-looking data, never a grey box).
- One sentence: "Laporan stok harian menunjukkan barang yang hampir habis sebelum pembeli datang."
- Two or three capability lines.
- Primary Button "Mulai Pro, Rp 99.000/bulan"; ghost Button "Nanti saja".

**`Paywall / Usage Limit / Default`**
- "Anda sudah memakai 50 dari 50 transaksi gratis bulan ini."
- Progress bar at 100% (Progress component), a two-row comparison "Gratis: 50 transaksi / bulan · Pro: tanpa batas".
- Primary "Upgrade ke Pro"; secondary "Hapus transaksi lama" or "Tunggu 1 Desember" when that's the real free path.

**`Paywall / Trial Ending / Default`**
- "Masa coba Anda berakhir 3 hari lagi (26 Nov)."
- What they keep and what stops: "Data Anda tetap aman. Laporan stok dan multi-kasir berhenti."
- What they did: "Anda sudah mencatat 312 transaksi dan 48 produk." (from the user's data; on the canvas use plausible values and annotate that Koda fills them).
- Primary "Lanjut dengan Pro"; secondary "Ingatkan besok"; tertiary "Pakai versi gratis".

Also draw the after-state: the upgrade confirmation (the feature opens right away, a receipt line, one pointer to the new feature), and the payment step kept short and in context (prefilled name and email; in Indonesia show the methods the brief names, such as QRIS, e-wallet or virtual account, as real brand icons per Taste rule 7).

## Mobile subscription apps

Consumer apps sold through the app stores (fitness, learning, religious study, scanners) run differently:
- **Placement:** onboarding, then a personalised result (a plan, a score, a first analysis), then the paywall while that result is on screen. A paywall on first launch asks for money with nothing to point at.
- **Hard paywall or freemium:** both are legitimate. The source cites RevenueCat's 2026 data: hard-paywall apps converted about five times more users in 35 days, with similar one-year retention. A free tier fits when it drives sharing; cap it daily so the limit is felt in the habit.
- **Plans:** lead with the yearly plan, the weekly or monthly equivalent as supporting text, the billed amount and period always clear ("Rp 12.000/minggu, ditagih Rp 599.000/tahun"). At most one shorter option beside it. Prices per storefront, not a converted number.
- **Trials:** a free trial on the yearly plan, with a reminder before it ends (draw the reminder notification). Trial length is the owner's decision to test; longer trials convert better but pay later.
- **One abandon offer:** if they close without starting a trial, one discounted yearly offer, shown once.
- **Store requirements** (draw them on the screen): price, billing period, trial length and "diperpanjang otomatis sampai dibatalkan"; links to Ketentuan and Privasi; "Pulihkan pembelian"; a visible close button.
- Store fees change by country and date; if the person asks, tell them to check current Apple and Google terms rather than quoting a figure.

## Never draw these (dark patterns)

- A hidden, delayed or tiny close button.
- Confusing plan selection (the cheaper plan unreadable, plans that look the same).
- Guilt copy on the no: "Tidak, saya tidak mau bisnis saya berkembang".
- Fake countdowns or "penawaran terbatas" with no real deadline.
- A paywall that blocks a critical flow (closing the day's cash, seeing an existing booking).
- A price revealed only at the last step.

If the person asks for one, explain the cost in trust and app-store rejection, and draw the honest version.

## Handoff notes for Koda

For each paywall frame annotate: trigger, frequency cap and cool-down, which values are live data, what "Nanti saja" does, and the events to track (paywall seen, upgrade tapped, upgrade done, dismissed). Ideas worth testing later, if the owner wants: trigger timing (after the aha vs at the feature tap), one plan vs a comparison, preview vs text only, yearly shown per month vs billed amount. Any countdown in a test must reflect a real deadline.

Adapted from coreyhaines31/marketingskills, skills/paywalls and its mobile-subscription-paywalls reference (MIT, Copyright (c) 2025 Corey Haines; mobile patterns from Blake Anderson's ai-app-playbook, MIT), via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
