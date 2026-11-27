# Native pen design system, phase 1: implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The native apps (desktop, Android, iOS) draw with frontend-new's pen tokens and Geist type, and every component in the web's foundations, actions, inputs, status, identity, agents and overlays sections has a matching Compose component.

**Architecture:** A new generator writes `NomiPenTokens.kt` from `frontend-new/src/lib/ds/tokens/tokens.json` and the `@theme` block of `frontend-new/src/app.css`. `NomiTheme` provides the pen colours and still builds a `MaterialTheme` from them, so untouched M3 code keeps compiling. The existing `Ds` facade in `ds/Foundation.kt` (web names on M3 roles today) is repointed at the pen tokens. The existing `Ds*` components are then redrawn to match their Svelte twins, and the missing ones are added.

**Tech Stack:** Compose Multiplatform (`native/shared/designsystem`), Kotlin, Node 22 (`node:test`) for the generator, Roborazzi desktop screenshots, `compose.desktop.uiTestJUnit4`.

**Spec:** `docs/superpowers/specs/2026-11-27-native-pen-design-system-design.md`

## Global Constraints

- **UX gate first.** `.claude/hooks/require-ux-contract.sh` denies edits under `native/shared/designsystem` and `native/shared/feature` until `.claude/state/active-work` names a work item whose `docs/ux/work/<slug>/40-ux-contract.md` has `status: verified`. Before Task 2, run `/ship native pen design system, phase 1` so ux-lead verifies the contract (slug `native-pen-phase-1`). Never use `NOMI_UX_BYPASS`.
- **Values come from the generated tokens or a named `const`.** No inline hex or bare numbers in components (memory: no magic numbers).
- **Compose names match the web.** Each Svelte component `X.svelte` → a Kotlin `DsX` (keep existing names: `Mascot`, `Kbd`, `KeyHint`, `AgentByline`, `CrewRow`, `Meter`, `SectionLabel`, `CheckMark`, `DialogPanel`, `MenuPanel`, `TooltipBubble`). The parameters are the Svelte props in camelCase, and events become `on…: () -> Unit` lambdas.
- **Tailwind → Compose mapping.** Use this when reading a Svelte class list:
  - 1 Tailwind unit = 4.dp, so `h-7.5` = 30.dp and `px-2.75` = 11.dp.
  - The `text-*` sizes and line heights come from `NomiPenType`, and `rounded-*` from `NomiPenRadius`.
  - Colour classes map to the `Ds` property of the same name (`bg-surface-2` → `Ds.surface2`).
  - `pointer-coarse:min-h-11` → `Modifier.minimumInteractiveComponentSize()` on Android and iOS only.
  - `hover:` → hover interaction state on desktop, pressed state on touch.
  - `focus-visible:outline-3 outline-focus` → the existing `nomiFocusRing` with `Ds.focus`.
- Commits follow Conventional Commits, scope `native`, with **no AI attribution lines**. Hooks are in `.githooks`.
- Branch `feat/native-pen-ds` (already cut from main). One PR at the end of the phase, based on main.
- Do not touch `frontend-new/`. It is the source.

## Review Focus

1. **Dark mode.** Every redrawn component in the dark theme should read with the dark `nm-*` values, not light values or leftover M3 roles. Task 3 pins `Ds` in both themes, and each component task's screenshot sheet renders light and dark.
2. **Feature screens still on M3** (`shared/app`, `feature/*`) should keep rendering legibly with the pen-derived `ColorScheme`: no unreadable onSurface-on-surface. Task 3 adds a contrast test over the bridge scheme.
3. **Touch targets.** A 30dp button or a 20dp icon button on a phone still needs a 44dp hit area. Tasks 4–7 each assert `minimumInteractiveComponentSize` behaviour through a touch-mode test.
4. **Long labels.** A long label, role or toast should ellipsise on one line where the web uses `truncate`/`whitespace-nowrap`, not wrap or overflow. This is pinned per component where the web test pins it (e.g. `CrewRow`).
5. **Missing fonts on a platform.** If the Geist resources fail to load, text should fall back to the platform sans and not crash. Task 2 tests that `penFontFamilies()` resolves on desktop. On Android and iOS this is covered by the CI build only; say so in the PR.

---

### Task 1: Pen token generator

**Files:**
- Create: `native/scripts/gen-pen-tokens.mjs`, `native/scripts/gen-pen-tokens.test.mjs`
- Create (generated): `native/shared/designsystem/src/commonMain/kotlin/ai/nomi/designsystem/generated/NomiPenTokens.kt`
- Modify: `.github/workflows/native.yml` (next to the `gen-tokens.mjs --check` step)

**Interfaces:**
- Produces, as Kotlin in package `ai.nomi.designsystem.generated`:
  - `data class PenColors(...)` with one `Long` (0xAARRGGBB) per `nm-*` colour, named in camelCase without the `nm-` prefix (`bg`, `surface2`, `borderStrong`, `text3`, `accentInk`, `dangerBg`, `focus`, `overlay`, `shadow`, `heat1`…`heat4`, `bgClear`, `codeKeyword`…, `diffAdd`, `diffDel`, `kbBg`, `kbKey`, `kbKeyAlt`, `peer1`, `peer1Bg`, `peer2`, `peer2Bg`).
  - `object NomiPenColors { val light: PenColors; val dark: PenColors }`.
  - `data class CrewColors(val base: Long, val bgLight: Long, val bgDark: Long, val inkLight: Long, val inkDark: Long)` and `object NomiCrewColors { val all: Map<String, CrewColors> }`, keyed `nomi`, `koda`, … `sora`.
  - `object NomiPenSpace` with `xs2 = 2`, `xs = 4`, `sm = 6`, `md = 8`, `lg = 12`, `xl = 16`, `xl2 = 20`, `xl3 = 24`, `xl4 = 32`, `xl5 = 48` (Int, dp).
  - `object NomiPenRadius` with `hair = 1`, `cell = 2`, `xs2 = 3`, `xs = 4`, `sm = 6`, `tile = 7`, `inner = 8`, `md = 10` (Int, dp).
  - `data class PenTypeRole(val size: Float, val lineHeight: Float, val trackingPx: Float)` and `object NomiPenType`. Its members are `display`, `heading`, `body`, `ui`, `caption`, `label`, `micro`, `nano`, `phoneTitle`, `phone`, `phoneSm`, `phoneLarge`, `phoneDisplay`; tracking is taken from `--text-*--letter-spacing` or the matching `--tracking-*`, else 0.
  - `object NomiPenFonts { const val SANS = "Geist"; const val MONO = "Geist Mono" }`.
- JS exports: `parsePenTokens(jsonText) → { colors: {name: {light, dark}}, numbers: {name: n}, strings: {name: s} }`, `parseThemeScale(cssText) → { type: {role: {size, lineHeight, tracking}}, radius: {name: px} }`, `emitPenKotlin(tokens, scale) → string`, plus the `--check` CLI flag (exit 1 when stale).

- [ ] **Step 1: Write the failing tests** in `gen-pen-tokens.test.mjs`:
  - `parsePenTokens` gives `colors['nm-bg']` = `{ light: 0xFFF6F6F4, dark: 0xFF0A0A0B }`.
  - `#C93C320D` parses to `0x0DC93C32` (RGBA → ARGB).
  - A single-value colour (`crew-nomi` `#61D192`) gives the same light and dark value.
  - `parseThemeScale(appCss).type.body` = `{ size: 15, lineHeight: 19, tracking: 0 }`, using `var(--nm-size-body)` resolved against tokens.json.
  - `.type.display.tracking` = `-0.4`, `.type.phoneLarge` = `{ size: 32, lineHeight: 42, tracking: -0.6 }`, `.radius.tile` = `7`.
  - The emitted Kotlin starts with `// Generated by native/scripts/gen-pen-tokens.mjs`, contains `val bg: Long` and `bg = 0xFFF6F6F4L`, and contains no `var(`.
- [ ] **Step 2: Run** `node --test native/scripts/gen-pen-tokens.test.mjs`. Expected: FAIL (module not found).
- [ ] **Step 3: Implement the generator.** Reuse `parseHex` by importing it from `./gen-tokens.mjs`. Read the paths relative to the repo root, as `gen-tokens.mjs` does. Use no dependencies.
- [ ] **Step 4: Run** the tests again. Expected: PASS. Then run `node native/scripts/gen-pen-tokens.mjs` and `node native/scripts/gen-pen-tokens.mjs --check`; the check should exit 0.
- [ ] **Step 5:** Add a CI step `node native/scripts/gen-pen-tokens.mjs --check` under the existing one. The existing `node --test native/scripts/*.test.mjs` already picks up the new test.
- [ ] **Step 6: Commit** `feat(native): generate pen tokens from frontend-new`.

### Task 2: Geist fonts

**Files:**
- Add: `native/shared/designsystem/src/commonMain/composeResources/font/geist_variable.ttf`, `geist_mono_variable.ttf`
- Create: `.../designsystem/theme/PenTypography.kt`
- Test: `.../desktopTest/kotlin/ai/nomi/designsystem/theme/PenTypographyTest.kt`

**Interfaces:**
- Produces: `@Composable fun penFontFamilies(): PenFamilies`, where `data class PenFamilies(val sans: FontFamily, val mono: FontFamily)`, with weights 400, 500 and 600 pinned on the variable wght axis (same pattern as `variableFamily` in `NomiTypography.kt`). Also `fun penTextStyle(role: PenTypeRole, family: FontFamily, weight: FontWeight = FontWeight.Normal): TextStyle`, mapping size → sp, lineHeight → sp, and trackingPx → sp, with the line-height style `LineHeightStyle(Center, Trim.None)`.

- [ ] **Step 1: Get the fonts.** Ask the user first, because this downloads files. The files are the OFL variable TTFs from the official repo `github.com/vercel/geist-font` (release assets `Geist[wght].ttf` and `GeistMono[wght].ttf`), renamed to the resource names above. Add `OFL.txt` beside them.
- [ ] **Step 2: Write the failing test.** `penTextStyle(NomiPenType.body, …)` has `fontSize == 15.sp`, `lineHeight == 19.sp` and `letterSpacing == 0.sp`. `penTextStyle(NomiPenType.display, …).letterSpacing == (-0.4).sp`. In a composition, `penFontFamilies().sans` is not `FontFamily.Default`.
- [ ] **Step 3: Run** `./gradlew :shared:designsystem:desktopTest --tests '*PenTypographyTest*'` from `native/`. Expected: FAIL (unresolved reference).
- [ ] **Step 4: Implement `PenTypography.kt`.**
- [ ] **Step 5: Run** the test again. Expected: PASS.
- [ ] **Step 6: Commit** `feat(native): bundle Geist and Geist Mono for the pen type`.

### Task 3: Theme and the `Ds` facade on pen tokens

**Files:**
- Modify: `.../theme/NomiTheme.kt`, `.../ds/Foundation.kt` (the `Ds` and `DsShapes` objects)
- Create: `.../theme/PenColorScheme.kt`
- Test: `.../desktopTest/kotlin/ai/nomi/designsystem/theme/PenThemeTest.kt`; re-record `src/desktopTest/snapshots/*.png`

**Interfaces:**
- Consumes: Task 1 `NomiPenColors`, `NomiPenRadius`, `NomiPenType`; Task 2 `penFontFamilies`, `penTextStyle`.
- Produces:
  - `NomiTheme.pen: PenColors`, through a new `LocalPenColors`. `NomiTheme(dark = …)` picks light or dark.
  - `fun PenColors.toColorScheme(dark: Boolean): ColorScheme` with this mapping:
    - primary=accent, onPrimary=accentInk, primaryContainer=accentBg, onPrimaryContainer=accent
    - background=bg, onBackground=text
    - surface=surface, onSurface=text, surfaceVariant=surface2, onSurfaceVariant=text2
    - surfaceContainerLowest=surface, Low=surface, Container=surface2, High=surface2, Highest=border
    - outline=borderStrong, outlineVariant=border
    - error=danger, onError=accentInk(light)/bg(dark), errorContainer=dangerBg, onErrorContainer=danger
    - tertiary=warning, tertiaryContainer=warningBg, secondary=text2, secondaryContainer=surface2
    - scrim=overlay
  - Every existing `Ds` colour property returns the pen colour of the same web name. Add the missing `Ds.focus`, `Ds.overlay`, `Ds.shadow`, `Ds.codeType`, `Ds.heat1`…`heat4`, `Ds.kbBg`, `Ds.kbKey` and `Ds.kbKeyAlt`. `Ds.dark` reads a `LocalPenDark`.
  - The `Ds` text styles use `penTextStyle` on Geist:
    - body, ui, caption, heading and display → sans
    - label and micro → mono
    - `labelCaps` = label + `--tracking-label` 0.6px
    - mono = caption on mono
    - title = ui at weight 500 (the web's `font-medium text-ui`)
  - Add `Ds.phone`, `Ds.phoneSm`, `Ds.phoneTitle`, `Ds.phoneLarge` and `Ds.phoneDisplay`.
  - `DsShapes`: xs = `NomiPenRadius.xs`, sm = `.sm`, md = `.md`; add `inner`, `tile`, `xs2`.
  - The crew colours: `fun crewColor(agent: String): Color`, `crewBg(agent)` and `crewInk(agent)`, which pick light/dark through `Ds.dark`.

- [ ] **Step 1: Write the failing tests.**
  - In `NomiTheme(dark = false)`: `Ds.bg == Color(0xFFF6F6F4)`, `Ds.accent == Color(0xFF1E9A58)`, `Ds.text3 == Color(0xFF7C7C84)`, `Ds.body.fontSize == 15.sp`, `DsShapes.sm` has corner size 6.dp.
  - In `NomiTheme(dark = true)`: `Ds.bg == Color(0xFF0A0A0B)`, `Ds.accent == Color(0xFF7BE3A4)`.
  - `crewBg("sora")` is light `0xFFE6E9FD` and dark `0xFF1C1F30`.
  - Bridge contrast: for both themes, the WCAG contrast of `onSurface`/`surface`, `onPrimary`/`primary` and `onSurfaceVariant`/`surface` in `toColorScheme` is ≥ 4.5. Write a small `contrast(a, b)` helper in the test.
- [ ] **Step 2: Run** `./gradlew :shared:designsystem:desktopTest --tests '*PenThemeTest*'`. Expected: FAIL.
- [ ] **Step 3: Implement** `PenColorScheme.kt`, the `NomiTheme` changes and the `Ds`/`DsShapes` changes. Keep `NomiTheme`'s signature: `accent` stays and is ignored for colours, with a KDoc line saying the pen has one accent. Keep `extendedColors` working by deriving `ground` = bg, `stage` = text, `onStage` = bg, and `actionBrush` = solid accent.
- [ ] **Step 4: Run** the test again. Expected: PASS. Then run the full suite: `./gradlew :shared:designsystem:desktopTest`. Expected: only the Roborazzi comparisons fail, with no logic failures. Re-record them with `./gradlew :shared:designsystem:recordRoborazziDesktop`. If that task doesn't exist, run with `-Proborazzi.test.record=true`, and write down which one worked. Open three of the new PNGs (`theme_light.png`, `theme_dark.png`, one `ds_*` sheet) and check they show the pen greys and green.
- [ ] **Step 5: Run** `./gradlew desktopTest testAndroidHostTest :desktopApp:test --continue` from `native/` (CI's command). Expected: PASS. Fix compile errors in `shared/app` and `feature/*` only where `Ds`/theme API names changed. Do not restyle feature screens in this phase.
- [ ] **Step 6: Commit** `feat(native): draw the design system with the pen tokens`.

### Tasks 4–7: Redraw and complete each section

Every section task follows the same steps. For each component:

- [ ] **a. Port the web test.** Read the Svelte file and its web test (`frontend-new/src/lib/ds/<section>/*.svelte.test.ts`). Copy each web test case into `desktopTest/kotlin/ai/nomi/designsystem/ds/<Section>Test.kt` with the same case name in backticks, read through the semantics tree. A computed colour becomes an assertion on the drawn colour, using `captureToImage()` pixel sampling at the node's centre or border. A size becomes `assertWidthIsEqualTo`/`assertHeightIsEqualTo`. A role becomes a semantics `Role`. Add the touch-target case from the Review Focus where the web has `pointer-coarse:min-h-11`.
- [ ] **b. Run** `./gradlew :shared:designsystem:desktopTest --tests '*<Section>Test*'`. Expected: FAIL. A redrawn component fails on its values; a new component fails as an unresolved reference.
- [ ] **c. Draw the component.** Use plain `foundation` primitives (`Box`, `Row`, `BasicText` via `DsText`, `clickable` with the `Role`), with values only from `Ds`, `DsShapes`, `NomiPenSpace` and named consts. Remove any M3 wrapper an existing `Ds*` uses (`NomiButton`, `ButtonDefaults`, `FilledIconButton`, `Switch`, …).
- [ ] **d. Run** the section test again. Expected: PASS. Then add the component to the section's sheet in `ComponentSheets.kt`/the gallery (light and dark), re-record that sheet, and look at the PNG.
- [ ] **e. Commit** once per section, e.g. `feat(native): actions drawn as on the web`.

**Task 4: foundations, actions.**
- Redraw: `DsIcon`, `DsButton`, `DsIconButton`, `Kbd`, `KeyHint`, `DsLink`.
- New: `DsGithubMark`, `DsPhonePill`.
- `VisuallyHidden` has no Compose form. Port its web test as a semantics-only `Modifier.visuallyHidden(label)` that sets `contentDescription` and draws nothing.
- Pinned values from `Button.svelte`:
  - height 30.dp, horizontal padding 12.dp (11.dp plus a 1.dp border for secondary and danger), radius `sm`, gap 6.dp, icon 14.dp, text `ui`
  - primary: bg accent, text accentInk, weight 500
  - inverse: bg text, text bg

**Task 5: inputs.**
- Redraw: `CheckMark`, `DsCheckbox`, `DsChip`, `DsSearch`, `DsSegmented` (+ `DsSegment`), `DsTabs` (+ `DsTab`), `DsTextField`, `DsToggle`.
- New: `DsSelect`, `DsPhoneSearch`, `DsComposer`.
- `DsComposer` is the web `inputs/Composer.svelte`. Leave `component/Composer.kt` for the feature screens until phase 2, and add a KDoc pointer between the two.

**Task 6: status, identity, agents.**
- Redraw: `DsStatus`, `DsTag`, `Meter`, `SectionLabel`, `DsAvatar`, `AgentByline`, `CrewRow`, `Mascot`.
- `Mascot` keeps its shapes and motion from `shape/`; only its colours move to `crewColor`/`crewBg`.
- Port `identity.svelte.test.ts`, `status.svelte.test.ts` and `Mascot.svelte.test.ts`. For example, Avatar is a 28.dp tile with mono initials, and CrewRow puts its role on one ellipsised line.

**Task 7: overlays.**
- Redraw: `DialogPanel`, `DsDialog`, `MenuPanel`, `DsMenu`, `DsToast`, `DsToastRegion`, `TooltipBubble`, `DsTooltip`.
- New: `DsMenuRow`, `DsModal`, `DsBottomSheet`, `DsActionSheet`, `DsSheetOption`, `DsSheetTiles`, `DsPhoneToastRegion`.
- Dialogs and sheets use Compose `Dialog`/`Popup` for the top layer. The scrim is `Ds.overlay`; the shadow is `Ds.shadow` at the web's blur, drawn with `Modifier.shadow` using spot and ambient colour = `Ds.shadow`.
- Port `topLayer.svelte.test.ts` as: Escape closes, focus moves into the panel and returns to the opener on close (desktop test with `performKeyInput`).

### Task 8: Phase check and PR

- [ ] **Step 1: Run** `./gradlew desktopTest testAndroidHostTest :desktopApp:test --continue`, `./gradlew :androidApp:assembleDebug` and `./gradlew iosSimulatorArm64Test --continue` (macOS) from `native/`. Expected: all PASS. Record any failure by name.
- [ ] **Step 2: Run** `node --test native/scripts/*.test.mjs`, `node native/scripts/gen-tokens.mjs --check` and `node native/scripts/gen-pen-tokens.mjs --check`. Expected: PASS, exit 0.
- [ ] **Step 3: Run the desktop app** (`./gradlew :desktopApp:run`) and the Android app on an emulator or the iOS simulator. Screenshot the gallery's phase 1 sections and one feature screen in both themes, and attach them to the PR.
- [ ] **Step 4: Open the PR** `feat(native): pen design system, phase 1` against main, with no attribution lines. The body lists:
  - what moved to pen tokens
  - the new components
  - which feature screens still use the Material bridge
  - the Review Focus item 5 caveat
