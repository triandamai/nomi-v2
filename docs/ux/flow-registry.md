# Flow registry — Nomi.pen

Every user flow, its frames, and where it stands. ux-lead keeps this current.
Frame ids are pen node ids in `Nomi.pen`. Seeded from the pen file on 16 Nov 2026.

Status: `gaps` · `designing` · `design-verified` · `building` · `shipped` · `build-verified`.
`designed` = frames exist but ux-lead hasn't run the gate on them yet.

## Chat
| Flow | Desktop dark | Desktop light | Mobile dark | Mobile light | Status |
|---|---|---|---|---|---|
| Ask → think → answer | ENb07 | NpWDs | FvlVs | r8Gz3 | designed |
| Clarify → plan | S3yL5 | g11Ew | GCvfD | tq7Ht | mobile: design-verified (gate 1, 17 Nov 2026 · P1/P2 → UXP-002) |
| Plan → approve → act | HU6nc | mjZPs | G1tK6L | EI3k6 | mobile: design-verified (gate 1, 17 Nov 2026 · P1/P2 → UXP-002) |
| Rich replies | y2Qxt2 | aaAGg | XO7DI | OkCH1 | designed |
| Like & dislike | HKYo1 | RzVHg | M0VaJC | mg56o | designed |
| Share & delete | TnscX | lSpCm | yoVD6 | Pjc23 | designed · steps 6–8 (share with files, G16) design-verified · UXP-001 P0 |
| Message controls | Cf6GO | AQ91t | ctP7Q | m2cy3I | designed |
| Attachments, voice & limits | Iyq9S | ORByl | FLwdb | Fpgnv | **design-verified · UXP-001 P0** (contract: `work/audit-chat-files/40-ux-contract.md`; P1/P2 gaps still open) |
| Manage chat & long threads | J3ks0 | tisDt | W3a8j | N2N1d | designed |
| Hand-offs, limits & memory | hiAY1 | j8DK1 | YuWzO | M9B8RO | designed |
| Approvals & background work | bWiWT | t16cU | xvntx | qhcNA | mobile: design-verified (gate 1, 17 Nov 2026 · steps 1–2 and 5 wait on backend · P1/P2 → UXP-002) |
| Errors & recovery | TpuRc | Nud6J | AyrHG | uuCam | mobile: design-verified except scenario 6 steps 4–6 (gate 1, 17 Nov 2026 · P1/P2 → UXP-002); file errors → UXP-001; scenario 6 file steps (desktop 5–7, mobile 4–6) design-verified · UXP-001 P0 |
| Shared link | b8Flw8 | Wgdu2 | Yyjkb | e0FI49 | designed · steps 5–7 (files private, Maya's file, files open) design-verified · UXP-001 P0 |

## Build
| Flow | Desktop dark | Desktop light | Mobile dark | Mobile light | Status |
|---|---|---|---|---|---|
| Sign in → live URL | o882RC | hifej | zM0xh | cyXXE | designed |
| Workspace tools | U16wU | omR64 | tJh5G | zdXZh | designed |
| Landing page | WUiji | pcnAT | yY8qf | oLJ3q | designed |
| Roll back, limits & hand-over | pwkEe | Qt5C0 | Wv14s | yYAZV | designed |
| Code, terminal & phone preview | uR6dq | kKWtu | r3IvZk | f48QD7 | designed |
| Invite & build together | TnwQT | O0wCuQ | L8dgcS | ZTTgV | designed |

## Projects
| Flow | Desktop dark | Desktop light | Mobile dark | Mobile light | Status |
|---|---|---|---|---|---|
| Act from the card | syolc | iRVjM | fGjF3 | d3vpxM | designed |
| Card menu | M9O0iU | C6TYml | T9IbH | aQ7W2 | designed |
| Detail panel | pgSon | GFmPN | G4Mwvd | bPkS8 | designed |
| Find & filter | V5R0N | pi4nB | owsMq | JIopm | designed |
| New project | DV63l | SoDkk | H4ngT2 | ATPK7 | designed |
| Import from GitHub | qJxWu | Xob5F | ktDHy | LjN47 | designed |
| Workspaces & people | rQHc2 | GFWya | F2Tuh | mc2EQ | designed |
| Loading, offline & errors | c6fFB | OSqdc | EkEA9 | U4wv6A | designed |

## Other
| Flow | Desktop dark | Desktop light | Mobile dark | Mobile light | Status |
|---|---|---|---|---|---|
| Renewal → Reminder | SFJZj | bl2TW | — | — | **gaps · no mobile** |
| Add crew member | lk7Ws | SV1y1 | — | — | **gaps · no mobile** |

## Key components for chat files
`Chat / Attach Menu` Wh2Sd · `Chat / Attachment / Uploading` o8o2x · `Chat / Attachment / Ready` iAYlM ·
`Chat / Attachment / Image` vFPxs · `Chat / User / With Files` ENNIK · `Chat / Composer Limit` U9ZWI ·
`Chat / Mention / File` Hoxfr · `Chat / Citation` C6xkT · `Chat / Sources` mrOeg · `Callout / Error` mhAYP ·
`Chat / Banner / Offline` i9fRJB · `Chat / User / Queued` JCfti · `Chat / User / Failed` KmbOy ·
`Chat / Attachment / Failed` BD98f · `Chat / Attachment / Rejected` xpYoV · `Chat / Attachment / Model Warning` rrjYM ·
`Callout / Warning` S971Sp · `Chat / Agent File` JSxfH · `/ Generating` vRi2e · `/ Expired` s1yQM · `/ Failed` xyZCb ·
`Chat / File Viewer` G534E (+ Loading HVM9y, Error K1phVf, Sheet l8KfHe, Image rCBA7) · `Mobile / File Viewer` GbEIf (+ b3lX47, nQ4Iq, snTHJ, LiYjQ) ·
`Chat / Share Dialog / Files` e1shsD · `/ Files Open` zWgDC (pen-only copies of eemXK; build one dialog)
