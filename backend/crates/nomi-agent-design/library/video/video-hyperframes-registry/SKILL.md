---
name: video-hyperframes-registry
description: "What the ~450 HyperFrames registry blocks and components offer by category (transitions, titles, captions, charts, maps, overlays, logos), which suit small-business videos, how to mount one, and who installs it by name. Read it before hand-building a named effect."
license: "Apache-2.0, Copyright 2026 HeyGen, Inc."
source: "https://github.com/heygen-com/hyperframes @574b5e3d3bbf526809d268a6728f9c5ce52e18bb"
---
# The HyperFrames registry

The registry is a catalogue of ready-made, deterministic pieces: 456 items at the commit read (210 blocks, 238 components and 8 example projects). Starting from one is faster and usually better than writing an effect yourself. Once installed, an item is ordinary source in the version, which you restyle with the brand's tokens, trim and retime.

## Who installs what

You don't fetch code. You name the item, and **Koda or the render worker installs it** before preview:

```bash
npx hyperframes catalog --query "<what the beat should do, in English>" --json   # search, nothing installed
npx hyperframes add <name> --json        # writes the files and prints the mount snippet
npx hyperframes add <name> --vars '{"title":"Es Kopi Pandan"}' --json
```

- Query in English even when the video is Indonesian. The index is English, and other scripts return "No searchable words".
- Narrow a search with `--type block|component` and `--tag <tag>` (tags below).
- `add` needs the network. Its dependencies install first.
- If nothing fits, say so and hand-build it. Describe the move in your own words.

## Blocks and components

| | Block | Component |
|---|---|---|
| What it is | A whole scene with its own timeline, size and duration | A snippet merged into your scene, driven by your timeline |
| Installs to | `compositions/<name>.html` | `compositions/components/<name>.html` |
| How you use it | Mount it with a host div | Copy its markup, `<style>` and script into your scene, following its comment header |

**Mounting a block.** Give the host div `data-composition-src="compositions/<name>.html"`, `data-composition-id="<name>"` (it must match the block's own id), `data-start`, a `data-duration` no longer than the block's own, `data-track-index`, and `data-width`/`data-height` equal to the **block's dimensions**. Set its copy and colours for this mount with `data-variable-values='{"title":"…","color-accent":"#C2552D"}'`, pointing colour and font variables at the brand's tokens. The block's timeline is seeked by the runtime, so don't reference it from your own GSAP code. Keep its ids as prefixed.

**Dimensions matter.** Most blocks are 1920×1080. In a 9:16 video, prefer a component (it inherits your canvas), a block made portrait (1080×1920), or position a landscape block deliberately inside the frame. Never stretch it.

**Beat-synced blocks** take hit times from `npx hyperframes beats --json`: the beat times minus the mount's `data-start`.

## Categories and when to use them

| Category (tag) | Count | Use it for | Good first picks |
|---|---|---|---|
| Text motion primitives (`typography`, `text`, `reveal`) | ~45 | Titles and lines landing well | `per-word-rise`, `soft-blur-in`, `kinetic-center-build`, `headline-slam`, `marker-highlight`, `typewriter`, `tracking-in`, `slot-machine-roll`, `ticker-takeover` |
| Title cards (`title-card`) | 15 | An opening or chapter title | `titlecard-calm`, `titlecard-lockup`, `hw-title`, `hw-write-title` (handwritten); `canopy-part-title` for a nature brand |
| Caption styles (`caption-style`) | 16 | Burned-in captions | `caption-pill-karaoke`, `caption-highlight` (TikTok-style), `caption-weight-shift` (calm), `caption-editorial-emphasis`, `caption-clip-wipe` |
| Transitions, CSS and GSAP (`transition`) | ~20 | Seams with a cause | `whip-pan-cut`, `iris-reveal`, `fade-through`, `match-cut`, `text-match-cut`, `morph-swap`, `torn-paper-rip`, `paper-collage-reveal` |
| Transition galleries | 13 | Reference for picking a style; don't embed as-is | `transitions-push`, `transitions-blur`, `transitions-scale`, `transitions-radial` |
| Shader transitions (`shader`) | 14 | One dramatic seam, at most 2 per video, tech kinds only | `flash-through-white`, `whip-pan`, `light-leak`, `cinematic-zoom`, `sdf-iris` |
| Lower thirds (`lower-third`) | 13 | Naming a speaker over footage | `lt-clean-bar`, `lt-soft-pill`, `lt-accent-underline`, `lt-kicker-name`; `news-ticker` |
| Social overlays (`social`, `overlay`) | ~12 | Ending on the business's own account | `instagram-follow`, `tiktok-follow` (both 1080×1920), `yt-lower-third`, `x-post`, `yt-comment-card` |
| Logo and end cards (`logo`, `cta`) | ~8 | The mark performed once, the action held | `logo-sting`, `logo-brand-close`, `logo-outro`, `yt-logo-intro`, `cta-lockup`, `cta-close`, `social-proof-card` |
| Product and UI props (`product-demo`, `ui-props`, `device`) | ~30 | Showing an app, site or phone screen | `device-frame-stage`, `browser-device-stage`, `screen-flow-carousel`, `ui-focus-zoom`, `cursor-zoom-follow`, `notification-stack`, `typed-prompt` |
| Cursors (`cursor`) | ~14 | A tutorial or demo | `oversized-cursor`, `touch-indicator` (mobile), `press-ripple`, `cursor-text-select` |
| Camera moves (`camera`) | ~14 | A push, pull or shake on a wrapper | `push-in`, `pull-back-reveal`, `pan-stations`, `camera-shake`, `focus-rack` |
| Compare (`compare`) | ~5 | Before and after | `before-after-wipe`, `comparison-split`, `grade-split-reveal` |
| Data and charts (`data`, `chart`) | ~15 | One real number or one chart | `number-wheel` (prices, counts), `chart-story`, `animated-bar-chart`, `data-chart`, `bar-chart-race`, `star-rating-fill`, `conic-progress-ring` |
| Maps (`map`, `geography`) | ~8 | A route or region | `nyc-paris-flight`-style route moves, `world-map`; the US and Spain maps only for those places |
| Image carousels (`images`, `gallery`) | 25 | Several photos in one beat | `carousel-path-*`, `carousel-circle-*`; `hw-frame` for one framed photo; `cinemagraph` (one living region in a still) |
| Handwritten (`handwritten`) | ~13 | A friendly, hand-made voice | `hw-arrow`, `hw-callout-circle`, `hw-underline`, `marker-checklist-card` |
| Texture and finish (`texture`) | ~20 | A film finish over the whole frame | `grain-overlay`, `vignette`, `light-sweep-pass`, `organic-light-leak-overlay` |
| Backgrounds (`background`) | ~30 | Grounds for tech kinds | `dynamic-grid`, `aurora-drift`, `mk-background`; mostly not for places |
| Chat and phone stories (`ad-template`, `chat`) | ~15 | A conversation or notes app playing out on a phone | `message-thread-reveal`, `notes-reveal`, `notification-cascade`, `chat-thread` |
| Code (`code`, `code-animation`) | ~35 | Developer content only | `code-typing`, `code-diff`, `code-terminal-run` |

Search finds what this table misses. `catalog --query` is the source of truth, and the tags listed are the ones that filter well.

## Picks for Nomi's usual asks

| Ask | Reach for |
|---|---|
| Reels promo for a cafe | A photo-led scene of your own, `per-word-rise` or `headline-slam` for the offer, `number-wheel` for the price, `whip-pan-cut` between dishes, `instagram-follow` or `cta-lockup` to close |
| Cafe TV menu | Your own scenes (one category each), `soft-blur-in` for names, `fade-through` between categories, `grain-overlay` at low strength if the brand is warm |
| Product launch | `device-frame-stage` or `browser-device-stage` with the real screens, `ui-focus-zoom`, `oversized-cursor`, `pull-back-reveal` to the result, `logo-brand-close` |
| Captions over a phone video | One `caption-style` component on the captions host; `lt-soft-pill` for the speaker's name |
| Animated logo | `logo-sting` (punchy), `logo-brand-close` (calm), `svg-stroke-trace` for a line mark drawn on |
| Testimonial or proof | `star-rating-fill` with the real rating, `social-proof-card`, `yt-comment-card` |

## What to avoid

- **Anything tagged `webgpu`** (40 items, such as `aurora`, `liquid-metal`, `ios26-liquid-glass` and `mesh-gradient`). They need a WebGPU browser the render worker doesn't run, so they come out blank. Avoid `html-in-canvas` and VFX blocks too unless the worker is known to support them.
- **Shader effects, glow, particles and 3D on a place, product or service** (Taste). Those kinds get photographs and type.
- **Other companies' branded UI as if it were the client's** (`chatgpt-exchange`, `claude-exchange`, `slack-notification-ad`, `heygen-avatar-promo-card` and the Apple-style showcase pieces). Use platform overlays only to point at the client's own account.
- **Demo content left in.** Replace every placeholder name, number, logo and photo with the brief's real ones, or with a labelled placeholder.
- Example projects (`product-promo`, `kinetic-type`, `warm-grain`, `swiss-grid` and others) are references for structure, not starting files for a client.

Adapted from hyperframes (Apache-2.0, HeyGen). Changed: rewritten for Nomi's video compositions.
