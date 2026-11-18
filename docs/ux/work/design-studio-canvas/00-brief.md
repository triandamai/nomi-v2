# Brief — design-studio-canvas

## The request
Evolve the Design Studio into a Figma-inspired 2D spatial design canvas where a person can:
1. Explore Sora's 2–4 style directions visually side-by-side on the canvas, give feedback to refine a direction, or pick one.
2. Inspect the Design System artboard and every App Flow page artboard side-by-side on a pan-and-zoom surface (`25%`–`200%`, `Fit`).
3. Click any `[data-design-id]` element directly on any artboard to drop a pin comment in an inline popover (without switching to a separate "Feedback mode" tab first) and batch-send revisions via a **Revise with Sora** bar.
4. Repeat revisions across versions (`v1`, `v2`, …) until satisfied, then approve the version so Sora creates the structured handoff and delegates to Koda.

## Who it's for
Anyone building an app with Rena, Sora, and Koda who wants to visually compare style directions, see the whole design system and screen flow at once, point at specific elements to request changes, and approve the final design before Koda writes code.

## Scope decisions (confirmed)
- **Target apps**: Both `frontend-new/` (`Nomi.pen` design system tokens) and `frontend/` (legacy SvelteKit app), backed by `nomi-server` (`GET /api/sessions/:id/design`).
- **Stage 1 (Direction Exploration)**: Automatically render visual style-tile artboards on the canvas directly from `propose_directions`' structured JSON (`palette`, `typography`, `layout`, `rationale`, `trade_offs`) without requiring extra HTML files per direction.
- **Stage 2 & 3 (Design System + App Flow)**: Render the Design System artboard alongside all App Flow page artboards (`index.html`, `menu/index.html`, etc.) on the 2D pan/zoom canvas, with device width presets (`390px` phone, `768px` tablet, `1280px` desktop).
- **Stage 4 (Direct Pin Feedback & Revise Bar)**: Clicking any `[data-design-id]` element on an artboard opens a floating pin popover right next to the element to save a note; open notes are queued into a bottom **Revise with Sora** bar that sends a single consolidated revision prompt to Sora.

## Design system & UX rules alignment
- **"An instrument, not a toy"** (`frontend-new/AGENTS.md`, `.claude/skills/nomi-ux-rules/SKILL.md`):
  - Canvas surface uses `bg` with a subtle `border` dot grid; artboard frames use `1px` `outline -outline-offset-1 outline-border` (`outline-accent` when selected).
  - Mint (`accent`) stays `< 10%` of the viewport: reserved for the single primary CTA (**Revise with Sora** when open notes exist, **Approve design** when ready) and the active selection outline/pin badge.
  - Typography: `Geist` (`font-sans`) for prose and labels; `Geist Mono` (`font-mono`) for all machine output (`v1`, `75%`, `390 × 844`, `#B5532A`, `data-design-id`, `index.html`, elapsed `00:14`).
- **All 5 required states**:
  1. **Empty**: Neither directions nor previews exist yet; explains that Sora proposes style directions after Rena's brief is confirmed.
  2. **Loading / Building**: When Sora is actively staging a preview (`design_preview_drafts` or active turn), displays real staged files and elapsed time (`font-mono`).
  3. **Error**: Surfaces unpublished preview warnings (`publish_error` + **Retry** button) and failed feedback/revision/approval calls with an inline retry action.
  4. **Populated**: 2D pan/zoom canvas with Stage 1 Direction Style-Tile Artboards, Stage 2 Design System Artboard, Stage 3 App Flow Artboards, inline pin popover, and right inspector rail.
  5. **Mobile (`390px`)**: Adapts the multi-artboard canvas into a horizontal artboard switcher with 44×44px coarse touch targets and a bottom sheet for element feedback and revision submission.
