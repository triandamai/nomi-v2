# Native apps on the frontend-new design system

## Why

frontend-new is the app now. The native apps (desktop, Android, iOS) should look and behave like it.
They still draw the old `frontend/` Material 3 look: `native/scripts/gen-tokens.mjs` reads
`frontend/src/lib/styles/material3.css`, and the components in `native/shared/designsystem` are
M3 twins. We are switching native to the pen look (decided 2026-11-27) and porting frontend-new's
components to it.

## What exists

- All three native apps share one Compose Multiplatform UI. iOS only hosts it
  (`native/iosApp/iosApp/ContentView.swift`), so one component set serves desktop and mobile.
- `native/shared/designsystem` has about 12k lines in `component/`, `ds/`, `shape/`, `brand/`,
  `navigation/`, `gallery/`, and `NomiTheme` wraps `MaterialExpressiveTheme`.
- 62 files outside the generated tokens read `MaterialTheme.` (31 in designsystem, the rest in
  `app`, `companion` and `feature/*`).
- frontend-new has about 450 components in 22 sections under `frontend-new/src/lib/ds/`, with
  Vitest browser tests that pin colours, sizes, fonts and behaviour.
- Pen tokens: `frontend-new/src/lib/ds/tokens/tokens.json` (colours light/dark, spacing, radii,
  type sizes, crew colours, code/diff/heat/peer colours). Phone type sizes and other theme values
  are in `frontend-new/src/app.css` (`--text-phone*`).
- Fonts: Geist and Geist Mono (`@fontsource-variable/geist*`, OFL). Native bundles Bricolage,
  Instrument Sans and JetBrains Mono today.

## Goals

- Native colours, type, spacing, radii and crew colours come from the pen tokens, generated, with
  the existing `--check` so CI fails when they drift.
- Every frontend-new component has a Compose twin with the same name, the same inputs and the same
  events, in `ai.nomi.designsystem.ds.<section>` (sections named as on the web).
- Phone-width windows use the `mobile` section and phone chrome; wider windows use the desktop
  shells (`shells`, `chrome`, `navigation`). The split follows the web's breakpoint.
- The native gallery mirrors `/design-system`, section by section.
- Feature screens (`feature/*`, `app`, `companion`) move onto the new components.

## Non-goals

- New backend APIs. A component the native features can't feed yet is ported and shown in the
  gallery, and wired later.
- SwiftUI or Android-View UIs. Everything stays in shared Compose.
- Keeping the Material 3 look as an option.

## Anti-goals

- Hand-typed colour or size values in components. Every value comes from the generated tokens or a
  named constant.
- Native layouts that drift from the web: copy and spacing follow the frontend-new component.

## Approach

Considered:
1. **Retheme then port, section by section (chosen).** Swap the token source and theme first, so
   every existing component changes look at once. Then port the missing components one section at
   a time, each with tests that copy the web tests' assertions.
2. Port all components into a new package, then switch over in one go. That's one huge change with
   nothing shippable until the end.
3. Keep M3 components and only change the colours. That's cheap, but the shapes, type and layouts
   would still be Material, not pen.

## Design

**Tokens.** `gen-tokens.mjs` reads `tokens.json` and the `@theme` values in `app.css` instead of
`material3.css`. It writes `NomiTokens.kt` with:
- `NomiPenColors` (light and dark), built from every `nm-*` colour;
- `NomiCrewColors` for each crew member's base, bg and ink;
- `NomiSpace`, `NomiRadius` and `NomiTypeSize` (including the phone sizes).

The agent shapes and motion keep their current source until frontend-new has its own.

**Fonts.** Bundle Geist and Geist Mono variable TTFs (OFL) in `composeResources/font`. Remove
Bricolage, Instrument Sans and JetBrains Mono once nothing uses them.

**Theme.** `NomiTheme` gives you:
- `NomiTheme.colors`, `.type`, `.space` and `.radius`;
- light/dark, following the system or a passed `dark`.

It still provides a `MaterialTheme` whose colour scheme maps from the pen colours. Any M3 widget not
yet replaced then looks close, and the 62 files keep compiling while they migrate.

**Components.** One Kotlin file per web section, split when it passes about 800 lines. Composable
names match the Svelte component names. Desktop tests in `desktopTest` mirror the web test for that
component: same cases, same expected colours and sizes, read through the Compose semantics tree.

**Phases.** Each phase is one PR from main, merged before the next starts:
1. Tokens, fonts, theme; foundations, actions, inputs, status, identity, agents, overlays; gallery
   skeleton.
2. chat, conversation, thread.
3. mobile, shells, chrome, navigation: the phone and desktop frames, plus the phone Home.
4. lists, inbox, profile, settings.
5. projects, build, design.
6. auth, admin. Then remove the M3 bridge and the old fonts.

Feature screens move onto a phase's components in that same phase.

## Process

This is user-facing, so each phase goes through the UX pipeline in `CLAUDE.md`. Phase 1 and the
section ports copy components that are already verified on the web, and the pen frames stay the
source. Each phase still needs an active verified contract before its UI edits, because the
`require-ux-contract.sh` hook guards the native UI paths too.

## Left to the builder

- Splitting files within a section.
- How the screenshot tests are organised.
- How the web's hover-only states map to touch (pressed states) and the breakpoint width on
  desktop windows.
