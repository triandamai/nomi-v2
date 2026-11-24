# Lint rules and other animation runtimes

`npx hyperframes lint` checks the structure as you write. `npx hyperframes check --json` is the gate before render: it runs lint, then a runtime pass, a layout audit and a contrast audit at sampled times. While any lint error remains, the layout and contrast audits don't run, and "0 samples" means nothing was checked. Each finding names its rule and carries a `Fix:` line. Contrast failures also carry a `suggestedColor`. A finding at a single sample, such as an entrance in flight, is information, not an error.

## Errors you will meet first

| Rule | What it means | Fix |
|---|---|---|
| `root_composition_missing_html_wrapper` | `index.html` is a bare fragment | Add doctype, `<html>`, `<head>`, `<body>` |
| `root_missing_composition_id`, `root_missing_dimensions` | The root lacks its id or size | Add `data-composition-id`, `data-width`, `data-height` |
| `missing_timeline_registry`, `gsap_timeline_not_registered` | No timeline registered | Register `window.__timelines["<id>"] = tl` |
| `timeline_id_mismatch` | The timeline key matches no `data-composition-id` | Use the same id |
| `missing_data_no_timeline` | A root or host with no timeline | Add `data-no-timeline`, or every render waits 45 s |
| `gsap_timeline_registered_before_async_build` | Registered, then tweens added later | Register at the end of the async callback |
| `missing_gsap_script` | GSAP is used but not loaded | Add the GSAP `<script>` before the animation script |
| `timed_element_missing_clip_class`, `timeline_element_missing_timing` | A timed element lacks `class="clip"` or its timing | Add `class="clip"`, `data-start`, `data-duration` |
| `clip_ends_past_root_duration` | A clip runs past the video's end | Raise the root's `data-duration` |
| `subcomposition_blanks_before_host` | The scene's slot ends before its content should | Make the host's slot span the time the scene shows |
| `standalone_composition_wrapped_in_template` | The root uses `<template>` | Only scene files use it |
| `subcomposition_style_outside_template` | Scene styles sit outside `<template>` | Move them inside |
| `subcomposition_root_styled_by_class` | The scene root is styled by class | Style it as `#root` |
| `duplicate_composition_id` | Two compositions share an id | Make the ids unique |
| `gsap_animates_clip_element` | A tween on a `.clip`'s display or visibility | Animate a child, or the clip's `opacity` |
| `gsap_css_transform_conflict` | CSS `transform` plus a GSAP transform | Set the start inside `fromTo` |
| `gsap_from_opacity_noop` | CSS `opacity: 0` plus `from({opacity:0})` | Remove the CSS opacity, or use `fromTo` |
| `gsap_infinite_repeat` | `repeat: -1` | Use a finite repeat count |
| `non_deterministic_code` | `Math.random`, `Date.now`, `gsap.utils.random` | Use a seeded PRNG or a precomputed constant |
| `requestanimationframe_in_composition` | A rAF clock | Draw from timeline time or the seek event |
| `video_missing_muted` | A `<video>` with neither sound flag | Add `data-has-audio="true"` or `muted` |
| `video_nested_in_timed_element` | The timing is on the wrapper | Time the `<video>` itself |
| `media_missing_id`, `duplicate_media_id` | Media without a unique `id` | Give each a unique id |
| `media_crossorigin_breaks_preview` | `crossorigin` on media | Remove it |
| `imperative_media_control` | `play()`, `pause()` or `currentTime` in script | Let the runtime drive the media |
| `base64_media_prohibited`, `placeholder_media_url` | Inline or fake media | Use an `assets/...` file |
| `missing_local_asset`, `audio_src_not_found` | The file isn't in the version | Save it under `assets/`, and check the path |
| `invalid_parent_traversal_in_asset_path` | `../` escapes the project | Keep paths inside the project |
| `font_family_without_font_face` | An unbundled font without `@font-face` | Ship the `.woff2` and declare it |
| `invalid_composition_variables_declaration`, `unknown_variable_binding` | Bad variable JSON, or a binding to an undeclared id | Fix the array, or declare the id |
| `caption_exit_missing_hard_kill` | A caption group can linger | `tl.set(group, {opacity:0, visibility:"hidden"}, end)` |
| `multiple_caption_tracks`, `caption_track_kind_missing` | Captions spread across hosts | Use one host with `data-track-kind="captions"` |
| `slideshow_invalid`, `slideshow_unresolved_ref` | A broken deck island | See `video-hyperframes-slideshow` |

## Warnings worth fixing

| Rule | Meaning |
|---|---|
| `css_transition_used` | A CSS `transition` runs on the browser clock. Put the change on the timeline. |
| `timeline_track_too_dense` | More than 3 timed elements in one file. Split into scenes. |
| `studio_missing_editable_id` | A visible element without a stable `id`, so the Studio can't target it |
| `root_dimensions_mismatch` | The viewport meta or body size doesn't match the root size |
| `composition_file_too_large` | Split the file. Past 16 MB the render refuses it (`COMPOSITION_TOO_LARGE`). |
| `font_face_local_only` | A `local()` font face doesn't exist on the worker |
| `negative_z_index` | Add `isolation: isolate` to the parent, or avoid negative z-index |
| `nested_clip_never_visible` | A nested clip's window misses its parent's window |

## Craft warnings from `check`

These are questions, not rules. Fix the finding, or keep it on purpose and say why. A deliberate hold can carry `data-hold`.

| Warning | Meaning |
|---|---|
| `line_unread` | A line isn't on screen long enough for its words |
| `dead_stretch` | More than 3 s with nothing changing |
| `text_below_floor` | Text under 2.6% of the frame's short side (28 px at 1080) |
| `first_frame_empty` | Frame 0 is blank, and it is the thumbnail on social players |
| `film_silent` | A film over 15 s with no sound. Fine for a muted TV loop; say so. |
| `outside_safe_zone` | Words outside the 9:16 social clear band |
| `sweep_static` | A video 3 s or longer with no visible change, or a still missing `data-no-timeline` |
| `text_occluded` | Text covered by another box. Use `data-layout-allow-occlusion` when the cover is intended. |

## Motion assertions

To have `check` verify intent, write `<scene>.motion.json` next to a scene file:

```json
{ "assertions": [
  { "kind": "appearsBy", "selector": "#intro-title", "bySec": 0.6 },
  { "kind": "before", "a": "#intro-title", "b": "#cta" },
  { "kind": "staysInFrame", "selector": ".price-card" } ] }
```

A failure reports `motion_appears_late`, `motion_out_of_order`, `motion_off_frame` or `motion_frozen`.

## Other runtimes

GSAP is the default. Use another runtime only when the material needs it.

| Runtime | Use for | How it stays seekable |
|---|---|---|
| CSS `@keyframes` | Simple decoration loops | Finite iteration count, `animation-fill-mode: both`, and `data-start` on the element. The runtime finds and seeks them. |
| WAAPI | Native keyframes | `element.animate(…, { fill: "both" })`, then `pause()`. Times are document time, so use `delay`. |
| Lottie / dotLottie | A logo animation exported from After Effects | `autoplay: false`, always set `loop`, and push the player to `window.__hfLottie`. Ship the player library and `.lottie` file in the project. |
| Canvas, Three.js, p5 | Generative or 3D work | Draw from `e.detail.time` in an `hf-seek` listener, and set `data-duration` on the root |

The seek protocol for a canvas:

```html
<script>
  function renderAt(t) { /* draw the frame for t seconds from state derived from t alone */ }
  window.addEventListener("hf-seek", (e) => renderAt(e.detail.time));
  renderAt(0);
</script>
```

For async work such as image decoding, call `e.detail.waitUntil(promise)` synchronously inside the listener. Heavy setup registers a `window.__hf.buildReady["<key>"]` promise.

A scene that draws from a function of time can also register `window.__hyperframes.registerFrameSource({ element, ready, render(sourceTime, signal), dispose })` on a host marked `data-no-timeline`. Repeated and out-of-order times must give the same frame.

Charts and map overlays are driven from a GSAP tween's `onUpdate`, never from the library's own transitions. Text that must fit a width uses `window.__hyperframes.fitTextFontSize(text, { fontFamily, fontWeight, maxWidth })`.

Adapted from hyperframes (Apache-2.0, HeyGen). Changed: rewritten for Nomi's video compositions.
