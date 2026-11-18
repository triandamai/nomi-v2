---
work: audit-chat-files
gate: 2
round: 2
result: pass
date: 16 Nov 2026
---

# Gate 2 — audit-chat-files

## Round 1

Scope: the P0 gaps of UXP-001 (G3, G4, G6, G7, G9, G11, G12, G16), from `20-pen-changes.md` parts A and B.
Every frame id listed there was opened with `Get` (text resolved through instances). Key frames were
screenshotted in dark and light. Dark and light copy were diffed frame by frame across all 41 new
Attachments steps, all Errors scenario 6 steps, and all Share & delete and Shared link steps:
the copy is the same in every pair.

**Verdict: `DESIGN_CHANGES_REQUIRED`.** Most of the work is strong. The P0 happy and failure states
exist in all four versions, and the copy matches the proposal. But four things block the gate:
- Numbers contradict each other on the same screen (G7) and across the flow (G11).
- A share link can promise masked amounts while it hands out the unmasked files (G16).
- Three new actions or async states have no result or failure frame (G6, G11).
- The mobile citation number points at a source that isn't in the list.

### Flows touched
| Flow | Desktop dark | Desktop light | Mobile dark | Mobile light |
|---|---|---|---|---|
| Attachments, voice & limits | Iyq9S (rows vnsf3, w5Zb1L, X4u51c) | ORByl (P5FFEI, wAtbD, IxzLO) | FLwdb (steps 9–29) | Fpgnv (steps 9–29) |
| Errors & recovery · scenario 6 | TpuRc (yhuBX, ymrwy, aF3PA) | Nud6J (F5kA4, EF25o, Ev7si) | AyrHG (S61SO, U6S0r, K2r8A) | uuCam (g2x0P, WUqGg, E1NuW) |
| Share & delete | TnscX (jWihN, eGAIJ) | lSpCm (UpQs9, C8R438) | yoVD6 (w7JuxE, GDoGz) | Pjc23 (bot0x, iKPaT) |
| Shared link | b8Flw8 (RCgbo, y8IKKB) | Wgdu2 (aoKlu, iBFXM) | Yyjkb (YWmux, TkJne) | e0FI49 (U1Ik68, H5R4YC) |

### Acceptance check per gap
| Gap | Result | Evidence (desktop dark · light · mobile dark · light) | Notes |
|---|---|---|---|
| G3 Upload failed | **pass** | Q7n2MM, N3Iyy · fhdWO, vTWuT · rrKfg, BhXcx, bITK4 · QAikj, APtff, QSA24; removed state yhuBX · F5kA4 · S61SO · g2x0P | The chip copy and Send hint are verbatim. Send is disabled while the chip is failed or retrying, and enabled after Remove. The "resume" copy depends on the backend: see M1. |
| G4 Not supported | **pass** | IvQAw · IpiJU · rEdL2 · WtYon; mixed pick ymrwy · EF25o · U6S0r · WUqGg | The copy is verbatim. No `.heic` anywhere (D2 holds). |
| G6 Model can't read | **fail** | deDgy, mDEso, JVUHc · XUc6r, CSvJi, xA6ae · sE5RV, DgnEc, WDeji · a7GWtv, W3w6z, L3Vmo | Fix 5 (model name), fix 9 (no capable model). |
| G7 Cost before read | **fail** | Z3efXY, HAczP · u34vwy, GAeyu · z274p9, ch6hu · EO53V, kin2h | The estimate line and the BYOK line are verbatim and mono. The meter contradicts the plan meter: fix 1. |
| G9 Can't be read | **fail** | a1cvaM, jHLXS, jEVTP, P4LQp · VT0HL, fXXsl, G5sHKu, B4Jkj · obOdo, g3LRC, tEAce, t1fDK · MbibE, jYJRJ, rDXIQ, DqRJc; recovery aF3PA · Ev7si · K2r8A · E1NuW | All four cases answer the readable files and name the bad one. The D3 estimate shows on scanned. Tone is wrong for partial and scanned: fix 3. |
| G11 Agent file | **fail** | LE0sS, HcwU7, FbWFY · Rj2cf, pEABq, RKhjR · XuJTt, p1Z7mN, m4EvW0 · Mm4pd, Uo6A1, Ct8u1 | Output is kept apart from uploads, with no progress bar and Stop. Missing: the row count (fix 2), Save to project result (fix 7), and a failed state (fix 8). |
| G12 Viewer & citation | **fail** (mobile only) | z3WgN, IOcPG, FZYBl, l91ie, Z4ulB · d5tGK, AvQxk, rN0MN, MaKLC, wOHkC · P3rVq, QVezi, HZNUG, VHx9h, LifdT · jWohS, aQBES, MSSmK, V42TL3, A6XcF | The desktop meets the AC: `[3]` opens page 3 and the 9 highlighted rows add up to `Rp 1.120.000`. On mobile the cite number is wrong: fix 4. |
| G16 Shared link files | **fail** | jWihN, eGAIJ, RCgbo, y8IKKB · UpQs9, C8R438, aoKlu, iBFXM · w7JuxE, GDoGz, YWmux, TkJne · bot0x, iKPaT, U1Ik68, H5R4YC | The default is private and the copy is verbatim. Masking and open files can both be on: fix 6. Maya's produced files are missing from the share: fix 10. |

### Completeness walk (whole flows, not only new frames)
| Moment | Covered by (frame id · step) | Gap |
|---|---|---|
| Entry | Iyq9S 1 attach menu; citation `[3]` z3WgN; Sources row P3rVq; Preview on card Z4ulB / LifdT; photo l91ie / VHx9h | Opening the viewer from a sent attachment chip, with no citation: allowed as a contract rule (G534E with `Cite Bar` RWrTx and `Footer` d0ylL hidden, opens at page 1). It needs no frame. |
| Before commit | G4 IvQAw; G6 deDgy / mDEso; G7 Z3efXY / HAczP; existing too-large TpuRc 1 | **G6 with no capable model:** the filtered menu has no empty state (fix 9). |
| In progress | G3 retry N3Iyy / bITK4; G11 generating LE0sS / XuJTt (real step + elapsed + Stop); viewer loading IOcPG / QVezi | — |
| Failure | G3, G4, G9 ×4, viewer error FZYBl / HZNUG | **G11 file generation fails:** no state, so the Generating card would never end (fix 8). |
| Result | G9 partial answers; G11 HcwU7; G6 sent-text-only JVUHc | **Save to project** has no result (fix 7). `Read as images`, `Skip` and `See skipped` follow the existing quick-reply pattern (K4uh0l posts the choice as the next user message, and Maya answers). That can go in the contract with no new frames. |
| After | G11 deleted FbWFY (`Make it again`); Replace file aF3PA; share G16 | **Shared link × Maya's file:** not drawn (fix 10). |
| Interrupts | Offline and signed-out stay as existing TpuRc scenarios; offline with files is G15 (P1) | Out of scope this round. |
| Access | not drawn (P1 G17) | The contract will carry focus order and live-region text. No frames needed. |

### Visual rules
- **Variables:** pass. A scan of 8,605 nodes (all new steps plus the new components) found 0 raw hex colours and 0 raw font families. Font sizes and radii in the screens are raw numbers, but they sit on the token scale (11/12/13/15/28 desktop, 13/15/16 mobile). That is the same convention as Mario's reference steps. The new components bind sizes to `$nm-size-*` (126 of 127).
- **Accent under 10%:** pass. Accent shows only on enabled Send, toggles on, the active citation (`$nm-accent-bg` + `$nm-accent`), and the highlight stripe in the viewer.
- **Mono for machine output:** pass. File names, sizes, codes (`locked · password`, `no_text · scanned`, `preview_timeout · 30 s`), token and Rp estimates, and model ids are all mono. Callout titles keep the file name in sans (designer's issue 6): accepted, because it's the component's title style.
- **States:** see fixes 7, 8 and 9.
- **Dark + light:** pass. The copy is identical in every pair, and the light screenshots (V5BsvH, OgGy7, St52v viewer) read correctly.
- **Mobile at 390:** pass. All screens are 390 × 844, and the viewer is a 390 × 756 sheet. The G3 actions moved to a sheet (BhXcx) because the chip is too narrow.

### Rulings on the designer's open issues
| # | Issue | Ruling |
|---|---|---|
| A1 | G4 blocks Send | **Confirmed.** Same rule as TpuRc 1 "File too large" and G3: a chip the user picked is never dropped silently. The hint `Remove kasir-setup.dmg to send.` is accepted. |
| A2 | G6 Send state | **Confirmed disabled.** "Warning, not error" is about tone (the file is fine), not about Send. A live Send would have to drop the photo silently or guess. Send stays disabled until the user picks `Switch model` or `Send text only`. ↵ does not send, and focus moves to `Switch model`. The fix line stands in for the `↵ send` hint. |
| A3 | Rejected helper placement | **Accepted:** helper line under the chips, as in the too-large pattern. |
| A4 | `deepseek-v3` vs `deepseek-flash` | **Keep `deepseek-v3`** as a separate text-only model. `deepseek-flash` reads the receipt photo in Iyq9S 3–4, so it can't be the text-only model. But use the mono id everywhere (fix 5). |
| A5 / B7 | Headers not updated | **Fix:** copy only (fix 11). |
| B1 | "Expired" vs D1 | **Copy accepted.** Under D1 the state means *deleted* (per-file delete). The contract calls the state `deleted`. The component name `Chat / Agent File / Expired` (s1yQM) can stay. Signed-URL expiry must never show: the client re-signs on click. If the source files were deleted too, `Make it again` is hidden, and the line reads `This file and the files it came from were deleted from the chat, so Maya can't make it again.` (contract text, no frame). |
| B2 | Masking doesn't reach inside files | **UX ruling:** the two options can't both be on (fix 6). A link that says "Shows Rp ••• and initials" must not also hand out the unmasked statement. That is a privacy promise we can't keep, and a helper sentence doesn't make it safe. Redacting files server-side later goes to Mario (M2). |
| B3 | Partial zip in red | **Change** (fix 3). Red means the user must fix the file (locked, damaged). Partial (6 of 9 read) and scanned (Maya can read it another way) are warnings. |
| B4 | Invented content | **Accepted:** `budget-rumah`, `nota-sep-2026.pdf`, `invoice-mei.xlsx` / `invoice-juni.xlsx`, `assets.zip`, `struk-belanja-okt.jpg`. `Rp 800` is accepted as an example, derived consistently from G7's ratio. The numbers must still agree with the existing frames (fixes 1, 2, 12). |
| B5 | Share dialog copies | **Accepted for the pen.** ui-engineer builds one dialog: eemXK plus an optional Files line and the `Let viewers open files` row, never a second dialog. Recommended, not blocking: next time eemXK is touched, add the Files row to it as a hidden slot, so e1shsD and zWgDC can become instances. |
| B6 | Callout titles in sans | **Accepted** (see Visual rules). |

### Fix list — round 2
Blocking unless marked *minor*. All four versions each time: desktop dark and light, mobile dark and light.

1. **G7 meter contradicts the plan meter.**
   - **Frames:** Z3efXY (screen reDw8) · u34vwy · z274p9 · EO53V.
   - **What's wrong:** the Meter says `82% left` / `≈ 80% left after this read`. On the same desktop screen, the sidebar says `2%` · `21.8K / 1M tokens this month`. TpuRc 3 (N6bqZ) says `82% … used`, not left. And 96.000 tokens on a 1M allowance is about 10 points, not 2.
   - **What it should be:** the same unit and direction as the plan meter: label `Nomi credits` · value `2% used` · caption `≈ 12% used after this read` (21,8K + 96K of 1M). Mobile uses the same values.
2. **G11 row count contradicts the existing flow.**
   - **Frames:** HcwU7, FbWFY, Z4ulB · pEABq, RKhjR, wOHkC · p1Z7mN, m4EvW0, LifdT · Uo6A1, Ct8u1, A6XcF, plus components JSxfH, l8KfHe and snTHJ.
   - **What's wrong:** `214 transactions` / `2 months, 214 rows` / `First 20 of 214 rows`. Iyq9S 3 (EYwDT) says the same statements hold 64 + 78 = **142** transactions, and Iyq9S 4 says `142 transactions`.
   - **What it should be:** `Done: 142 transactions from September and October…` · `CSV · 18 KB · 2 months, 142 rows` · `First 20 of 142 rows` (desktop) · `First 12 of 142 rows` (mobile). Update the step captions that say 214 (Z4ulB, LifdT and their light twins).
3. **G9 tone for partial and scanned.**
   - **Frames:** P4LQp, jHLXS · B4Jkj, fXXsl · t1fDK, g3LRC · DqRJc, jYJRJ.
   - **What's wrong:** `Callout / Error` (danger header) on an answer that mostly succeeded (6 of 9 read), and on a file Maya can still read as images.
   - **What it should be:** a new variant `Callout / Warning` built from mhAYP, with `$nm-warning` for the icon and title, `$nm-warning-bg` for the fill, and the same border token pairing as the `rrjYM` chip. Icon `triangle-alert`. Locked (a1cvaM…) and damaged (jEVTP…) stay `Callout / Error`. Copy is unchanged.
4. **G12 mobile cites a source that isn't there.**
   - **Frames:** P3rVq · jWohS.
   - **What's wrong:** the caption `Tapping [3] in Sources…` and the cite chip `3`. Mobile Sources (p1Z7mN, obOdo, oDmXb) lists only `[1] bca-sep-2026.pdf · p. 3, 8` and `[2] bca-oct-2026.pdf · p. 2, 7`.
   - **What it should be:** the cite chip `1`, and the caption `Tapping [1] in Sources opens bca-sep-2026.pdf full screen at page 3, cited rows highlighted. Swipe down to go back.` The cite bar `Food in September · p. 3, 8` stays.
5. **G6 model named two ways on one screen.**
   - **Frames:** deDgy, mDEso, JVUHc · XUc6r, CSvJi, xA6ae · sE5RV, DgnEc, WDeji · a7GWtv, W3w6z, L3Vmo, plus component rrjYM.
   - **What's wrong:** the chip and the composer say `deepseek-v3`, but the warning and the sent note say `DeepSeek V3`.
   - **What it should be:** `deepseek-v3 can't see images` (mono) on the chip, and `Sent without struk-belanja-nov.jpg · deepseek-v3 can't see images` in the bubble.
6. **G16 masked link with unmasked files.**
   - **Frames:** dialogs eGAIJ · C8R438 · GDoGz · iKPaT and component zWgDC; viewers y8IKKB · iBFXM · TkJne · H5R4YC.
   - **What's wrong:** `Hide amounts and names` and `Let viewers open files` are both on, and the viewer meta reads `amounts and names hidden · files open to viewers`.
   - **What it should be:** the two toggles can't both be on.
     - With files open on, `Hide amounts and names` shows Toggle / Off CAJtO, with the helper `Off while viewers can open files. Amounts inside files can't be hidden.`
     - The files-open helper becomes `Anyone with the link can open and download these 2 files, with every amount in them.` Mobile: `…these 2 files, amounts included.`
     - Turning Hide back on turns files open off.
     - The viewer pages show the reply unmasked. Meta: `expires in 7 days · files open to viewers` (mobile `files open`). Note: `M. lets viewers open these files.`
     - The default dialog (jWihN · UpQs9 · w7JuxE · bot0x) is unchanged.
7. **G11 `Save to project` has no result.**
   - **Frames:** new step after HcwU7 · pEABq · p1Z7mN · Uo6A1.
   - **What it should be:** the card's `Save to project` becomes a quiet confirmation `Saved to budget-rumah` with a check icon and an `Open` link, and the path in mono, `budget-rumah/pengeluaran-sep-okt-2026.csv`. If the save fails, use the Toast / Error eT6bB `Couldn't save to budget-rumah · storage_full` with `Try again` (copy can live in the step caption).
8. **G11 file generation can't fail.**
   - **Frames:** new component `Chat / Agent File / Failed` and a new step after LE0sS · Rj2cf · XuJTt · Mm4pd (the answers row has one free slot).
   - **What's wrong:** the Generating card (vRi2e) has Stop but no end state when writing fails.
   - **What it should be:** the card keeps the name, meta `CSV · not made` (mono), and the line `Maya couldn't finish this file. Nothing was saved, and the two statements are still in this chat.` Show a mono reason code, e.g. `write_failed`, and the actions `Try again` and `Ask Maya why`. No danger fill on the whole card: just the icon and code in `$nm-danger`.
9. **G6 when no model can read the file.**
   - **Frames:** a new step after mDEso · CSvJi · DgnEc · W3w6z.
   - **What it should be:** the filtered menu or sheet with zero matches:
     - title `No model here can see images`;
     - line `deepseek-v3 and 2 others hidden · can't see images` (mono);
     - row `Add one in Models & providers` → link;
     - secondary `Send text only`.
     This is the BYOK-only case the proposal names ("some BYOK models are text-only").
10. **G16 × G11: Maya's file in a shared reply.**
    - **Frames:** new step in Shared link after RCgbo · aoKlu · YWmux · U1Ik68, plus the dialog Files line.
    - **What it should be:**
      - **Files private:** Maya's card (JSxfH) under `MADE BY MAYA` shows the name and `CSV · 18 KB · private` with a lock icon. It has no `Download`, `Preview`, `Save to project` or `Make it again`.
      - **Files open:** only `Download` and `Preview`, never `Save to project` or `Make it again`. This can be stated in the caption.
      - **Dialog:** the dialog's Files line lists produced files too, e.g. `bca-sep-2026.pdf · bca-oct-2026.pdf · pengeluaran-sep-okt-2026.csv` when that reply has one.
11. *Minor — copy only.* Update the flow headers and scenario heads that no longer describe their frames:
    - Iyq9S vg1Um / ORByl UXuwv / FLwdb q7ER6k / Fpgnv uxzqU (eyebrow and description gain upload failed, unsupported, model warning, cost, can't read, Maya's files, viewer);
    - TpuRc scenario 6 head D9zrJ and its Nud6J / AyrHG / uuCam twins (`… · UPLOAD · TYPE · UNREADABLE` and one sentence);
    - the Share & delete and Shared link headers (file privacy).
12. *Minor — content.*
    - **Frames:** z3WgN · d5tGK (component G534E page 3).
    - **What's wrong:** page 3 of 12 opens with `01/09 SALDO AWAL` and runs 01/09–27/09. Meanwhile the CSV (l8KfHe) puts rent on `02/09/2026`, between those rows, while Sources cites rent on page 8.
    - **What it should be:** drop the `SALDO AWAL` row so page 3 runs `03/09 … 27/09`. Mobile P3rVq already leaves it out. Date the CSV's rent rows `28/09/2026` and `28/10/2026` so they fall on the cited pages. Then re-check the food total `Rp 1.120.000` (it doesn't change).

Not to change: everything else in the scope, including the Errors scenario 6 steps, G3 / G4 copy, the G7 BYOK variant, the G9 locked and damaged cases, the viewer loading / error / lightbox / sheet, and the default share dialog.

### Three lenses
**Product.** The user always learns which file, what happened and what to do next, and the answer never pretends it read what it didn't. G6 "Sent without…" and G9 "isn't in this answer yet" are exactly right. The remaining product gaps are dead ends: Save to project with no result (fix 7), a file that never finishes (fix 8), a filtered menu with nothing in it (fix 9). Partial success also reads as a failure (fix 3).

**Business.** G7 is the cost-honesty promise, and right now its own numbers disagree on one screen (fix 1). That is the kind of mismatch that creates support tickets. G16 is the trust line: as drawn, the owner can produce a "masked" link that hands out the full bank statement (fix 6). The produced CSV, which holds every transaction, isn't covered by the share rules at all (fix 10).

**Tech.** Checked read-only in `backend/crates/nomi-server/src/app.rs`, `routes/attachments.rs`, `routes/shares.rs`, `routes/model_options.rs` and `frontend-new/BACKEND_TODO.md`. This feeds the contract's "Exists?" column next round.

| Need | Today |
|---|---|
| Per-file upload status | **Partial.** `POST /api/attachments` (streamed, one request per file) returns `status: ready \| pending`. Errors are plain strings (`413 file is too big`, `507 storage is full`) with no reason codes. **No resumable upload:** there is no Content-Range or offset handling (see M1). |
| Rejection reasons | **Missing.** `GET /api/attachments/limits` gives sizes and counts only (`max_file_bytes`, `max_files`, …), not accepted types. The server sniffs the MIME type but has no `unsupported_type` code. G4's client-side check needs the accepted-type list from config. |
| Model capability matrix | **Exists.** `GET /api/llm/model-options` returns `capabilities` including `vision` (`nomi-llm/src/catalog.rs` `Capability::Vision`). There is no PDF / max-pages capability yet. |
| Pre-send estimate | **Missing.** There is no estimate route. |
| Extraction result per file | **Partial.** The attachment `status` and `details.scanned` exist (`nomi-agent-core/src/attachments.rs`). There is no `locked`, `damaged` or `partial` code, and no read / skipped counts for zips. |
| Output file storage + signed URLs | **Missing.** There are no agent-output file routes. `GET /api/attachments/:id/content` and `/preview` serve uploads only, behind auth, not signed. |
| Share-link file visibility | **Missing.** Shares have `mask` and `anyone` but no files flag, and `GET /api/shared/:token` returns messages without attachments. BACKEND_TODO also says **names aren't masked server-side yet**, although the existing dialog promises initials (see M3). |

### Open for Mario (product decisions, not design failures)
- **M1 · Resumable uploads.** G3 copy promises a resume (`Resuming · 41% · 2,1 of 5 MB`, Retry meta `picks up at 41%`), but the backend can only restart.
  - **Choice:** build resumable uploads (the copy stays), or accept a restart. With a restart, the copy becomes chip `Uploading again · 0% · 0 of 5 MB`, Retry meta `starts the upload again`, and the step 9 caption changes to match.
  - **Until you decide:** the contract will carry both, gated on the backend.
- **M2 · Redacting files in share links.** Fix 6 makes masking and open files mutually exclusive. Masking amounts *inside* PDFs and CSVs server-side would let both be on. That is a future backend investment, not this round.
- **M3 · "Hide amounts and names" promises more than the server does.**
  - **The gap:** names aren't masked server-side (BACKEND_TODO), and file names can carry names too (e.g. `gaji-siti-rahayu.pdf`).
  - **The choice:** either drop "and names" and "initials" from the helper until the server can do it, or mask file names when Hide is on.
  - This was there before this round, in the existing Share flow, but G16 makes it more visible.
- **M4 · The unit for Nomi credits.** G7 shows an Rp price (`≈ Rp 1.900`) next to an allowance measured in tokens (`1M tokens this month`). Should the user see Rp, a percentage of the allowance, or both? The proposal's Rp example stays until you decide.

## Result
`DESIGN_CHANGES_REQUIRED`: fixes 1–10 are blocking, and 11–12 are minor and go in the same pass. Round 2 of 3 goes back to
pen-designer. The flow registry is unchanged: Attachments stays `designing · UXP-001`.

## Round 2

Scope: fixes 1–12 from round 1, checked against `20-pen-changes.md` `## Round 2 fixes`. Then the
completeness test was re-run on the four touched flows, and the visual rules were checked on the new nodes.

Method:
- Every frame id in the round-2 notes was opened with `Get`, with instances resolved and disabled nodes skipped.
- 28 dark/light pairs were diffed text by text: the copy is identical. The one difference is `Report this page` vs `Report` in Shared link. That is pre-existing in Mario's step 1 (v72jgW vs oGMsN).
- Close-ups were screenshotted: menus, sheet, cards, toasts, callouts and the files-open dialog.

**Verdict: `PASS`.** All ten blocking fixes and both minor fixes are in, in all four versions.

### Fix check
| # | Fix | Result | Evidence (desktop dark · light · mobile dark · light) |
|---|---|---|---|
| 1 | G7 meter | **pass** | Z3efXY · u34vwy · z274p9 · EO53V: `Nomi credits` · `2% used` · `≈ 12% used after this read`. Sidebar on the same screen: `2%` · `21.8K / 1M tokens this month`. Same unit and direction. |
| 2 | 142 rows | **pass** | HcwU7 · pEABq · p1Z7mN · Uo6A1; JSxfH / l8KfHe / snTHJ; captions Z4ulB / wOHkC / LifdT / A6XcF. A search of all touched flows and components finds no `214`. |
| 3 | Warning tone | **pass** | bYazl / IJOvD · WPhDu / ELvfx · p03cO / d1h2Wl · LohED / Y494On → S971Sp (`$nm-warning-bg` header, `$nm-warning` icon and title, `triangle-alert`). Locked and damaged are still mhAYP (l1deS, G7l2b, w50Nd, t4nrBr, zbhHo, TiEHK, dpDnc, xL1lW). |
| 4 | Mobile cite | **pass** | P3rVq · jWohS: chip `1`, caption `Tapping [1] in Sources…`. The page now runs 04/09–27/09 and the 9 rows add up to `Rp 1.120.000`. |
| 5 | Model id | **pass** | rrjYM F76Rd mono `deepseek-v3 can't see images`; sent notes aw1Dn / XWmYt / aBbKs / UNt4f mono; focus ring XHDLj / nkhFC (`$nm-focus`, 2 px outer); the `↵ send` hint is disabled in deDgy / XUc6r. |
| 6 | Mask ⊕ files open | **pass** | Dialogs eGAIJ · C8R438 (zWgDC b8yeZM off) · GDoGz f0k0mX off · iKPaT YtrSI off: helpers verbatim and the preview unmasked. Viewers y8IKKB · iBFXM · TkJne · H5R4YC: unmasked, meta `files open to viewers` / `files open`, note `M. lets viewers open these files.` Defaults jWihN · UpQs9 · w7JuxE · bot0x are unchanged. |
| 7 | Save to project | **pass** | Saved: mXjAV · yck4A · rmDzX · bIrng. Failed: htKXj · g2zGl · RA75h · j9gTY, toasts S1wMx / adA8H / WDMsd / mr0II. The code stays mono in the detail line: accepted. |
| 8 | Generation fails | **pass** | M9fOk · j8tL6 · Zh65u · ecvv2, component xyZCb: `CSV · not made`, `write_failed` in `$nm-danger`, no danger fill, `Try again` · `Ask Maya why`. |
| 9 | No capable model | **pass** | alcxf (er95B) · j3PeSH (sn1jx) · s10RHA (Sc4iM) · g1YGU: title, mono hidden line, `Add one in Models & providers` → link, and `Send text only`. |
| 10 | Maya's file in a link | **pass** | Viewer gpoF2 · a9Wfi · BBZuX · tRiRa: lock + `CSV · 18 KB · private`, no actions, and the note covers Maya's file. Dialog O35v6r · TuPME · RCHEQ · x92w7: the Files line lists all 3. The files-open card is stated in the caption, as allowed. |
| 11 | Headers | **pass** | vg1Um / UXuwv / q7ER6k / uxzqU; D9zrJ / SQSJJ / diWmy / ZTS3M (`… · UPLOAD · TYPE · UNREADABLE`); the Share & delete (W7xLO / FBZat / EJjLq / rTPZU) and Shared link (SO38P / HAnbK / H5PKN / IEbNc) headers now cover file privacy and Maya's file. |
| 12 | Statement content | **pass** | G534E page 3 runs `03/09 … 27/09`, and `SALDO AWAL` (uS5r0) is disabled. The CSV dates rent `28/09/2026`. The highlight total is unchanged. |

### Numbering and captions
- **Desktop Attachments:** steps 8–31 across rows vnsf3 / P5FFEI (8–15), w5Zb1L / wAtbD (16–23) and X4u51c / IxzLO (24–31). The labels are sequential, and no caption refers to another step by number.
- **Mobile:** FLwdb and Fpgnv each have 33 steps, with labels 1–33 in order. The captions match their screens: step 29 says `[1]`, and step 33 says `142`.
- **Share & delete:** steps 1–8 (new 8: O35v6r / TuPME / RCHEQ / x92w7).
- **Shared link:** steps 1–7 (new 6: gpoF2 / a9Wfi / BBZuX / tRiRa; old 6 is now 7).
- **Errors scenario 6:** desktop 1–7 and mobile 1–6, unchanged.

### Completeness re-run (whole flows)
| Moment | New state | Covered by | Verdict |
|---|---|---|---|
| Before commit | No capable model | alcxf / s10RHA. `Add one…` leaves for Settings. | Contract rule: the draft and chip survive the trip to Settings. No frame needed. |
| Failure | Generation fails | M9fOk / Zh65u. `Try again` → Generating; `Ask Maya why` → quick-reply pattern. | Covered. |
| Failure | Stop during Generating | — | Contract text: xyZCb with `stopped` and its line, plus `Chat / Stopped` bTCPw. No new component. |
| Result | Saved | mXjAV / rmDzX, `Open` → project file | Covered. |
| Failure | Save fails | htKXj / RA75h, toast + `Try again`, card keeps the button | Covered. |
| After | Maya's file in a shared link: private, open, deleted, failed | gpoF2 / BBZuX (private); caption (open) | Contract text: a deleted or failed produced file shows in a link as name + `no longer stored` / `not made`, with no actions. |
| After | Files line with 3 files | O35v6r / RCHEQ | Contract: the helper count follows the list (`these <n> files`). |
| Access | Focus for the new states | XHDLj / nkhFC (desktop G6) | Contract: focus and live regions for every new state. |

No new P0 gap: every new state ends in a recovery action or a stated rule.

### Visual rules (new nodes)
Scanned: 2,140 visible nodes across the 24 new steps, xyZCb, S971Sp, r17Ay and the G6 steps.
- **Variables:** pass. 0 raw colours; fonts are only `$nm-sans` (512) and `$nm-mono` (175).
- **Accent under 10%:** pass. Accent shows only on enabled Send, the mobile-web `Open App` chrome (as in the existing YWmux) and the share-sheet Copy tile. The disabled Send uses `$nm-surface-2`, the same as the approved Q7n2MM.
- **Mono for machine output:** pass. Codes (`write_failed`, `storage_full · …`), paths, model ids and the hidden-models line are all mono.
- **States:** pass. Every new async state has an end state.
- **Dark + light:** pass. The copy is identical, and the light roots carry `theme: {mode: light}`.
- **Mobile at 390:** pass. The toast and sheet fit, and the saved row wraps to its own line.
- **Layout:** no clipping found by `ctx.problems` on the new steps.

### Rulings on the designer's notes
| Note | Ruling |
|---|---|
| No focus ring on phone pills | **Accepted.** Touch has no visible focus, and VoiceOver and TalkBack draw their own. The contract requires the standard `$nm-focus` ring on `:focus-visible` at every width, for hardware keyboards. No frame needed. |
| Code in the toast's detail line, not the title | **Accepted.** It keeps the code mono and the title plain. |
| Mobile saved confirmation on its own row | **Accepted.** 358 px can't hold it inline. |
| Upload chips hidden on the CSV request (Shared link 6) | **Accepted.** It matches Iyq9S step 23, where the request had no attachments. |
| Desktop no-model menu widened to 392 px | **Accepted.** |
| Sources cite October rent on page 2 | **No change for this gate.** The statement and CSV contents are sample data, and the cited-page total is correct. If touched again, cite October rent on a late page (e.g. `page 10`). The contract carries this as N1, together with the CSV preview's September rows (17 rows, food ≠ `Rp 1,75 jt`), which don't match the 64 transactions. Non-blocking: the build uses live data. |
| `Expired` = deleted; "sources gone" text only | **Confirmed** (round-1 ruling B1). The text is in the contract. |
| One dialog (eemXK + Files row) | **Confirmed.** Contract note N3. |

### Open for Mario
M1–M4 stand as written in round 1. They don't block the design and are carried in `40-ux-contract.md` § Open items.

## Result
`PASS`. `40-ux-contract.md` is written with `status: verified`. The flow registry is updated:
- Attachments → `design-verified · UXP-001 P0`;
- Errors & recovery, Share & delete and Shared link carry notes.
