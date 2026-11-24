# The 36 deck themes

All 36 html-ppt themes, each with its mood, palette roles in hex, source fonts and best use on Nomi decks. A theme is only a set of token values. You treat it as a mood and a palette to start from, then write it as Nomi tokens.

## How to use a theme

**Role mapping** (html-ppt variable → Nomi token, written in the deck as a CSS custom property without the `$`):

| html-ppt | Nomi token | Column below |
|---|---|---|
| `--bg` | `--color-background` | ground |
| `--surface` | `--color-card` | card |
| `--text-1` | `--color-foreground` | text |
| `--text-2` | `--color-muted-foreground` | muted |
| `--accent` | `--color-primary` | primary |
| `--accent-ink` | `--color-primary-foreground` | on-primary |
| `--accent-2`, `--accent-3` | `--color-chart-2`, `--color-chart-3` | accent 2 · 3 |
| `--good` / `--warn` / `--bad` | `--color-success` / `--color-warning` / `--color-destructive` | not listed |
| `--radius` | `--radius-lg` | r |
| `--font-display` / `--font-sans` | `--font-heading` / `--font-body` | fonts |

Every theme leaves `--good`, `--warn` and `--bad` at the defaults: #1aaf6c, #f5a524 and #e0445a.

**Rules for using a theme**

- **Fonts are the source's, kept as a record.** Replace Inter, Space Grotesk, Noto Sans SC and Noto Serif SC (the Chinese faces aren't needed for Indonesian) with a `taste-type` pairing of the same character. Keep Archivo Black and JetBrains Mono only where Taste allows: a display face of character, or mono on tech decks. Oswald and Latin Modern Roman aren't loaded by the source either and fall back.
- **Violet and lavender accents are off on non-tech decks** (Taste): catppuccin, dracula, tokyo-night's second accent, pitch-deck-vc's second accent, vaporwave, y2k-chrome, rainbow-gradient, rose-pine. If the mood fits, keep the structure and swap the accent.
- **Effect themes** (marked ✱) carry blur, glow, gradient spots, scanlines or grid textures. These vanish in editable PPTX, so the deck must read without them. Their translucent cards (`rgba` surfaces) need a solid tint for export.
- **Dark themes** (ground darker than #333) are for tech-kind decks only.
- **Client colours replace the primary.** Re-check primary against on-primary for contrast (AA, 4.5:1). Each source theme was tuned so its pair passes.

## Light and calm

| Theme | Mood → best use | ground · card | text · muted | primary / on-primary | accent 2 · 3 | fonts (display / text) | r |
|---|---|---|---|---|---|---|---|
| minimal-white | Restrained, near-black accent → internal reports, reviews, serious topics | #ffffff · #ffffff | #0c0d10 · #55596a | #111216 / #ffffff | #3b3f4a · #6b6f7a | Inter / Inter | 14 |
| editorial-serif | Magazine serif on warm paper → brand story, a bakery or restaurant's story deck | #faf7f2 · #ffffff | #1b1410 · #5c4a3e | #8a2a1c / #ffffff | #c97a4a · #1b1410 | Playfair Display / same | 4 |
| soft-pastel | Soft macaron → salon, beauty, kids, light consumer topics (the pink primary is pale: use dark text on it) | #fdf7fb · #ffffff | #3a1f33 · #6b4d62 | #f49bb8 / #0b1024 | #b5d5f0 · #f7d08a | Inter / Inter | 24 |
| xiaohongshu-white | White with warm red, serif titles → Instagram carousels, lifestyle, promo posts | #fffdfb · #ffffff | #1a1210 · #4f3a32 | #ff2742 / #0b1024 | #ff7a90 · #ffb38a | Noto Serif SC / Noto Sans SC | 20 |
| solarized-light | Low glare for long viewing → workshops, training days | #fdf6e3 · #ffffff | #073642 · #586e75 | #268bd2 / #0b1024 | #2aa198 · #d33682 | Inter / Inter | 10 |
| catppuccin-latte | Developer-friendly light → tech talks (violet primary: tech only) | #eff1f5 · #ffffff | #4c4f69 · #6c6f85 | #8839ef / #ffffff | #1e66f5 · #ea76cb | Inter / Inter | 14 |
| arctic-cool | Cool blue and slate, rational → finance, analysis, reports to a bank | #f2f6fb · #ffffff | #0e1f33 · #3a5778 | #1e6fb0 / #ffffff | #17b1b1 · #6f8aa6 | Inter / Inter | 14 |

## Bold and statement

| Theme | Mood → best use | ground · card | text · muted | primary / on-primary | accent 2 · 3 | fonts | r |
|---|---|---|---|---|---|---|---|
| sharp-mono | Pure black and white, hard shadows → manifestos, a bold launch line | #ffffff · #ffffff | #000000 · #1a1a1a | #000000 / #ffffff | #000000 · #ff2200 | Archivo Black / same | 0 |
| neo-brutalism | Thick outlines, hard shadows, yellow → a startup pitch with attitude, a youth brand | #fffef0 · #ffffff | #000000 · #222222 | #ffd400 / #0b1024 | #ff5ca8 · #3a7cff | Archivo Black / Space Grotesk | 6 |
| bauhaus | Geometry, primary red, yellow and blue → design talks, a creative studio | #f4efe3 · #ffffff | #111111 · #333333 | #e03c27 / #000000 | #f4c430 · #1d4eaf | Archivo Black / Space Grotesk | 0 |
| swiss-grid ✱ | Grid lines, strict type, red → serious design-led decks, architects, agencies | #ffffff · #ffffff | #111111 · #444444 | #d6001c / #ffffff | #111111 · #888888 | Inter (Helvetica feel) / same | 0 |
| memphis-pop ✱ | Pop dots, big type → youth brands, events, collaborations | #fef6e8 · #ffffff | #111111 · #333333 | #ff3d8b / #0b1024 | #37c2d7 · #ffcc00 | Archivo Black / Space Grotesk | 10 |

## Cool and dark (tech only)

| Theme | Mood → best use | ground · card | text · muted | primary / on-primary | accent 2 · 3 | fonts | r |
|---|---|---|---|---|---|---|---|
| catppuccin-mocha | Soft dark → long developer talks | #1e1e2e · #313244 | #cdd6f4 · #a6adc8 | #cba6f7 / #0b1024 | #89b4fa · #f5c2e7 | Inter / Inter | 14 |
| dracula | Classic purple-pink dark → code-heavy talks | #282a36 · #343746 | #f8f8f2 · #bdbde0 | #bd93f9 / #0b1024 | #ff79c6 · #8be9fd | Inter / Inter | 12 |
| tokyo-night | Blue night → tech sharing, infrastructure (html-ppt's presenter default) | #1a1b26 · #24283b | #c0caf5 · #a9b1d6 | #7aa2f7 / #0b1024 | #bb9af7 · #7dcfff | Inter / Inter | 12 |
| nord | Cold Nordic blue → cloud, infra, SaaS | #2e3440 · #3b4252 | #eceff4 · #d8dee9 | #88c0d0 / #0b1024 | #81a1c1 · #b48ead | Inter / Inter | 12 |
| gruvbox-dark | Warm retro dark → terminal and open-source audiences | #282828 · #3c3836 | #ebdbb2 · #d5c4a1 | #fabd2f / #0b1024 | #fe8019 · #b8bb26 | Inter / Inter | 6 |
| rose-pine | Muted rose dark → design-meets-dev talks | #191724 · #26233a | #e0def4 · #c4b8d8 | #ebbcba / #0b1024 | #c4a7e7 · #9ccfd8 | Inter / Inter | 14 |

## Warm and vibrant

| Theme | Mood → best use | ground · card | text · muted | primary / on-primary | accent 2 · 3 | fonts | r |
|---|---|---|---|---|---|---|---|
| sunset-warm | Orange, coral, amber, upbeat → warung, cafe, food brands, award or celebration decks | #fff7ef · #ffffff | #2a160a · #6b4630 | #e36a2d / #0b1024 | #f2a341 · #d94860 | Inter / Inter | 18 |

## Effect-heavy

| Theme | Mood → best use | ground · card | text · muted | primary / on-primary | accent 2 · 3 | fonts | r |
|---|---|---|---|---|---|---|---|
| glassmorphism ✱ | Frosted glass on colour spots → an Apple-style tech launch | #0b1024 · rgba(255,255,255,.06) | #f2f4ff · #c3c8e6 | #7dd3fc / #0b1024 | #c084fc · #f0abfc | Inter / Inter | 22 |
| aurora ✱ | Aurora gradient, blur → tech cover and close slides | #06091c · rgba(255,255,255,.05) | #e8f0ff · #b4c4e4 | #5ef2c6 / #0b1024 | #7aa2ff · #c984ff | Inter / Inter | 20 |
| rainbow-gradient | White with flowing rainbow accent → festive and celebration slides (gradient: PDF only) | #ffffff · #ffffff | #0c0d10 · #4d5162 | #ff4d8b / #0b1024 | #7a5cff · #36b6ff | Inter / Inter | 16 |
| blueprint | Engineering blueprint, mono → system architecture | #0b3a6f · rgba(255,255,255,.06) | #e8f3ff · #b8d4f0 | #ffffff / #0b1024 | #aee1ff · #ffd27a | JetBrains Mono / same | 2 |
| terminal-green ✱ | Green-screen glow → CLI, security, retro hacker talks | #030a04 · #0a1b10 | #8cff9a · #4bd17a | #00ff88 / #0b1024 | #67ffd0 · #b6ff6b | JetBrains Mono / same | 4 |

## Light and professional

| Theme | Mood → best use | ground · card | text · muted | primary / on-primary | accent 2 · 3 | fonts | r |
|---|---|---|---|---|---|---|---|
| corporate-clean | White, navy, conservative borders → board reports, B2B sales, banks, insurance, a KUR loan pitch | #ffffff · #ffffff | #0a2540 · #425466 | #0a2540 / #ffffff | #1d4ed8 · #64748b | Inter / Inter | 6 |
| pitch-deck-vc | White, blue accent, big space → fundraising and investor meetings (drop the violet-pink gradient) | #ffffff · #ffffff | #0b0d12 · #4a5270 | #0070f3 / #ffffff | #7928ca · #ff4ecb | Inter / Inter | 14 |
| academic-paper | Paper white, serif, ink, blue links → research, training material, formal reports | #fdfcf8 · #ffffff | #0a0a0a · #333333 | #1a3a7a / #ffffff | #0a0a0a · #8a1a1a | Latin Modern Roman / same | 0 |
| japanese-minimal | Ivory, vermilion, huge space, serif → craft and artisan stories, brand refresh, a quiet restaurant | #fafaf5 · #ffffff | #1a1a18 · #5c564c | #d93a2a / #ffffff | #1a1a18 · #c9a961 | Noto Serif SC / Inter | 0 |
| engineering-whiteprint ✱ | White graph paper, navy ink, mono → system design, technical white papers | #ffffff · #ffffff | #0a1e46 · #3a4a6a | #0a1e46 / #ffffff | #1e5ac4 · #c42a10 | JetBrains Mono / Inter | 0 |

## Bold and editorial

| Theme | Mood → best use | ground · card | text · muted | primary / on-primary | accent 2 · 3 | fonts | r |
|---|---|---|---|---|---|---|---|
| magazine-bold | Cream, huge serif, orange spot → feature stories, brand magazines, a roastery | #f5efe2 · #fbf6e8 | #0a0a0a · #2a2a2a | #ea5a1a / #0b1024 | #0a0a0a · #c42a10 | Playfair Display / Inter | 0 |
| news-broadcast | White, red bar, condensed capitals, hard shadow → announcements, data bulletins (use sentence case anyway) | #ffffff · #ffffff | #0a0a0a · #3a3a3a | #e11d2d / #ffffff | #0a0a0a · #ffd100 | Oswald / same | 0 |
| midcentury | Cream, mustard, teal, burnt orange, sharp geometry → furniture, homeware, retro brands | #f3ead8 · #f9f2e0 | #201810 · #5a4830 | #d4902a / #0b1024 | #2a7a7f · #c7502a | Playfair Display / Inter | 2 |
| retro-tv ✱ | Warm cream, CRT scanlines, amber → nostalgia, 80s and 90s themes, an old family warung's story | #f5ecd7 · #fbf5e2 | #2a1a08 · #6b4a22 | #e67e14 / #0b1024 | #c73a1f · #f2b544 | Playfair Display / Inter | 10 |

## Dramatic

| Theme | Mood → best use | ground · card | text · muted | primary / on-primary | accent 2 · 3 | fonts | r |
|---|---|---|---|---|---|---|---|
| cyberpunk-neon ✱ | Pure black, neon pink, cyan and yellow, glow → hacker culture, gaming events | #000000 · #0f0f1a | #f5f7ff · #b4b8d4 | #ff2bd6 / #0b1024 | #00f0ff · #f9f871 | JetBrains Mono / Inter | 6 |
| vaporwave ✱ | Deep purple, pink and cyan haze → music, street art (violet: tech or gaming only) | #1a0938 · rgba(255,255,255,.06) | #fdf0ff · #d4a8e8 | #ff6ec7 / #0b1024 | #00f5ff · #ffd166 | Space Grotesk / same | 18 |
| y2k-chrome ✱ | Silver chrome, rainbow accent, big radius → Gen-Z fashion (tightest contrast pair in the set, 4.54:1) | #dfe4ec · rgba(255,255,255,.72) | #1a1f2e · #4a536a | #8a5cff / #0b1024 | #3ccfd8 · #ff84c4 | Space Grotesk / same | 26 |

## Quick picks for Nomi's people

| Deck | Start from |
|---|---|
| Warung, cafe, bakery menu | sunset-warm, editorial-serif, magazine-bold |
| Salon, spa, beauty | soft-pastel, japanese-minimal, xiaohongshu-white |
| Bengkel, workshop, trades | news-broadcast, sharp-mono, corporate-clean |
| Pitch to an investor or bank | corporate-clean, pitch-deck-vc, swiss-grid |
| Monthly report | minimal-white, arctic-cool, corporate-clean |
| Workshop, training | solarized-light, academic-paper, midcentury |
| Instagram carousel | xiaohongshu-white, soft-pastel, magazine-bold |
| Software or AI talk | tokyo-night, nord, engineering-whiteprint |

Adapted from html-ppt-skill (MIT). Changed: rewritten for Nomi's decks.
