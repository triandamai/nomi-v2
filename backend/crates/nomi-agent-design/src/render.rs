//! Pure-Rust OpenPencil (`.pen`) visual renderer and layout inspector for Sora.
//!
//! Parses one or more `.pen` files, resolves design variables, themes, and reusable component
//! instances (`ref` + `descendants`), runs an auto-layout pass matching OpenPencil's flex rules,
//! emits an SVG, and rasterizes it to PNG via `resvg` so Sora can visually inspect every screen,
//! component, or element she creates before showing it to the user.

use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;
use std::sync::{Arc, OnceLock};

use base64::Engine as _;
use resvg::{tiny_skia, usvg};
use serde_json::Value;

/// Maximum pixel dimension (width or height) of a rendered PNG screenshot so images stay crisp
/// for vision models while keeping payload size fast and compact.
const MAX_RENDER_PX: f64 = 1600.0;
/// Padding around a cropped element or overview canvas, in logical pixels.
const CROP_PADDING: f64 = 24.0;
/// Maximum nesting depth when expanding `"type": "ref"` component instances.
const MAX_REF_DEPTH: usize = 12;

/// Summary of one top-level screen/artboard in the `.pen` document.
#[derive(Debug, Clone, PartialEq)]
pub struct ScreenInfo {
    pub id: String,
    pub name: String,
    pub width: f64,
    pub height: f64,
    pub theme: String,
}

/// Bounding box of a laid-out element inside the rendered target.
#[derive(Debug, Clone, PartialEq)]
pub struct ElementBox {
    pub id: String,
    pub kind: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// The rendered PNG screenshot plus structured layout metadata and visual diagnostics.
#[derive(Debug, Clone)]
pub struct RenderedScreenshot {
    pub png_base64: String,
    pub width: u32,
    pub height: u32,
    pub target_label: String,
    pub target_kind: &'static str,
    pub theme: String,
    pub screens: Vec<ScreenInfo>,
    pub elements: Vec<ElementBox>,
    pub diagnostics: Vec<String>,
}

/// Cached system font database so repeated screenshots rasterize in milliseconds.
fn fontdb() -> Arc<usvg::fontdb::Database> {
    static DB: OnceLock<Arc<usvg::fontdb::Database>> = OnceLock::new();
    Arc::clone(DB.get_or_init(|| {
        let mut db = usvg::fontdb::Database::new();
        db.load_system_fonts();
        db.set_sans_serif_family("DejaVu Sans");
        db.set_serif_family("DejaVu Serif");
        db.set_monospace_family("DejaVu Sans Mono");
        Arc::new(db)
    }))
}

/// Renders a `.pen` preview (one or more `(path, content)` files) into a PNG screenshot.
///
/// - `target`: `None` or `Some("all")` renders the full canvas overview of all top-level screens;
///   otherwise matches a top-level screen by `id` or `name`, or any nested element by `id`.
/// - `theme_override`: optional `"light"` or `"dark"` override for token resolution.
pub fn render_pen_screenshot(
    files: &[(&str, &str)],
    target: Option<&str>,
    theme_override: Option<&str>,
) -> Result<RenderedScreenshot, String> {
    let doc = MergedPen::parse(files)?;
    let mut diagnostics = Vec::new();

    // Lay out every top-level frame, carrying theme forward as OpenPencil does.
    let mut active_theme = "light".to_string();
    let mut laid_out_screens: Vec<LaidOutScreen> = Vec::new();
    for raw_screen in &doc.children {
        if raw_screen.get("enabled").and_then(Value::as_bool) == Some(false) {
            continue;
        }
        if let Some(t) = extract_theme(raw_screen) {
            active_theme = t;
        }
        let screen_theme = theme_override.unwrap_or(&active_theme).to_string();
        let ctx = ResolveCtx {
            vars: &doc.variables,
            components: &doc.components,
            theme: &screen_theme,
        };

        let id = node_str(raw_screen, "id").unwrap_or("screen").to_string();
        let name = node_str(raw_screen, "name").unwrap_or(&id).to_string();
        if raw_screen.get("fill").is_none() {
            diagnostics.push(format!("Screen `{id}` has no `fill`; Studio will paint its default background."));
        }

        let expanded = expand_node(raw_screen, &ctx, 0, &mut diagnostics);
        let screen_x = resolve_number(expanded.get("x"), &ctx, &mut diagnostics).unwrap_or(0.0);
        let screen_y = resolve_number(expanded.get("y"), &ctx, &mut diagnostics).unwrap_or(0.0);
        let node = layout_node(&expanded, &ctx, None, None, &mut diagnostics);

        laid_out_screens.push(LaidOutScreen {
            id,
            name,
            x: screen_x,
            y: screen_y,
            theme: screen_theme,
            node,
        });
    }

    if laid_out_screens.is_empty() {
        return Err("the .pen canvas has no visible top-level frames to screenshot".to_string());
    }

    // Check for overlapping top-level screens.
    for i in 0..laid_out_screens.len() {
        for j in (i + 1)..laid_out_screens.len() {
            let a = &laid_out_screens[i];
            let b = &laid_out_screens[j];
            if rects_overlap(a.x, a.y, a.node.w, a.node.h, b.x, b.y, b.node.w, b.node.h) {
                diagnostics.push(format!(
                    "Top-level frames `{}` and `{}` overlap on the canvas; separate them by at least 120px.",
                    a.id, b.id
                ));
            }
        }
    }

    let screens_summary: Vec<ScreenInfo> = laid_out_screens
        .iter()
        .map(|s| ScreenInfo {
            id: s.id.clone(),
            name: s.name.clone(),
            width: s.node.w,
            height: s.node.h,
            theme: s.theme.clone(),
        })
        .collect();

    let requested = target.map(str::trim).filter(|t| !t.is_empty() && !t.eq_ignore_ascii_case("all"));

    let (svg, logical_w, logical_h, target_label, target_kind, theme, elements) = match requested {
        None => render_overview_svg(&laid_out_screens),
        Some(query) => {
            if let Some(screen) = find_screen(&laid_out_screens, query) {
                render_screen_svg(screen)
            } else if let Some((screen, elem, abs_x, abs_y)) = find_element(&laid_out_screens, query) {
                render_element_svg(screen, elem, abs_x, abs_y)
            } else {
                let available: Vec<String> = laid_out_screens.iter().map(|s| format!("`{}` ({})", s.id, s.name)).collect();
                return Err(format!(
                    "no screen or element `{query}` found in the preview. Available screens: {}",
                    available.join(", ")
                ));
            }
        }
    };

    let (png_bytes, px_w, px_h) = rasterize_svg(&svg, logical_w, logical_h)?;
    let png_base64 = base64::engine::general_purpose::STANDARD.encode(png_bytes);

    // Deduplicate diagnostics while preserving order.
    let mut seen = std::collections::HashSet::new();
    diagnostics.retain(|d| seen.insert(d.clone()));

    Ok(RenderedScreenshot {
        png_base64,
        width: px_w,
        height: px_h,
        target_label,
        target_kind,
        theme,
        screens: screens_summary,
        elements,
        diagnostics,
    })
}

fn find_screen<'a>(screens: &'a [LaidOutScreen], query: &str) -> Option<&'a LaidOutScreen> {
    screens
        .iter()
        .find(|s| s.id.eq_ignore_ascii_case(query) || s.name.eq_ignore_ascii_case(query))
}

fn find_element<'a>(screens: &'a [LaidOutScreen], query: &str) -> Option<(&'a LaidOutScreen, &'a LaidOutNode, f64, f64)> {
    for screen in screens {
        if let Some((node, ax, ay)) = find_node_recursive(&screen.node, query, 0.0, 0.0) {
            return Some((screen, node, ax, ay));
        }
    }
    None
}

fn find_node_recursive<'a>(node: &'a LaidOutNode, query: &str, ox: f64, oy: f64) -> Option<(&'a LaidOutNode, f64, f64)> {
    let ax = ox + node.x;
    let ay = oy + node.y;
    if node.id.eq_ignore_ascii_case(query) {
        return Some((node, ax, ay));
    }
    for child in &node.children {
        if let Some(found) = find_node_recursive(child, query, ax, ay) {
            return Some(found);
        }
    }
    None
}

fn render_overview_svg(screens: &[LaidOutScreen]) -> (String, f64, f64, String, &'static str, String, Vec<ElementBox>) {
    // Compute canvas bounding box.
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    let label_h = 28.0;

    for s in screens {
        min_x = min_x.min(s.x);
        min_y = min_y.min(s.y - label_h);
        max_x = max_x.max(s.x + s.node.w);
        max_y = max_y.max(s.y + s.node.h);
    }

    let pad = 40.0;
    let w = (max_x - min_x + pad * 2.0).max(120.0);
    let h = (max_y - min_y + pad * 2.0).max(120.0);

    let mut defs = String::new();
    let mut body = String::new();
    let mut id_counter = 0usize;
    let mut elements = Vec::new();

    // Subtle Studio canvas backdrop so light and dark artboards both stand out clearly.
    let _ = writeln!(body, r##"<rect x="0" y="0" width="{w:.1}" height="{h:.1}" fill="#E6E4DF"/>"##);

    for s in screens {
        let sx = s.x - min_x + pad;
        let sy = s.y - min_y + pad;
        let _ = writeln!(
            body,
            r##"<text x="{sx:.1}" y="{:.1}" font-family="DejaVu Sans, sans-serif" font-size="14" font-weight="600" fill="#52525B">{}</text>"##,
            sy - 10.0,
            escape_xml(&format!("{} ({})", s.name, s.id))
        );
        // Subtle artboard elevation border.
        let _ = writeln!(
            body,
            r##"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="none" stroke="#00000022" stroke-width="1"/>"##,
            sx - 0.5,
            sy - 0.5,
            s.node.w + 1.0,
            s.node.h + 1.0
        );
        emit_node_svg(&s.node, sx, sy, &mut defs, &mut body, &mut id_counter);
        elements.push(ElementBox {
            id: s.id.clone(),
            kind: "screen".into(),
            x: s.x,
            y: s.y,
            width: s.node.w,
            height: s.node.h,
        });
    }

    let svg = wrap_svg(w, h, &defs, &body);
    let theme = screens.first().map(|s| s.theme.clone()).unwrap_or_else(|| "light".into());
    (svg, w, h, format!("All screens ({})", screens.len()), "overview", theme, elements)
}

fn render_screen_svg(screen: &LaidOutScreen) -> (String, f64, f64, String, &'static str, String, Vec<ElementBox>) {
    let w = screen.node.w.max(1.0);
    let h = screen.node.h.max(1.0);
    let mut defs = String::new();
    let mut body = String::new();
    let mut id_counter = 0usize;

    // Default Studio backdrop if the screen omitted `fill`.
    let _ = writeln!(body, r##"<rect x="0" y="0" width="{w:.1}" height="{h:.1}" fill="#FFFFFF"/>"##);
    emit_node_svg(&screen.node, 0.0, 0.0, &mut defs, &mut body, &mut id_counter);

    let mut elements = Vec::new();
    collect_boxes(&screen.node, 0.0, 0.0, &mut elements, 60);

    let svg = wrap_svg(w, h, &defs, &body);
    (svg, w, h, format!("{} (`{}`)", screen.name, screen.id), "screen", screen.theme.clone(), elements)
}

fn render_element_svg(
    screen: &LaidOutScreen,
    elem: &LaidOutNode,
    _abs_x: f64,
    _abs_y: f64,
) -> (String, f64, f64, String, &'static str, String, Vec<ElementBox>) {
    let w = elem.w.max(1.0) + CROP_PADDING * 2.0;
    let h = elem.h.max(1.0) + CROP_PADDING * 2.0;
    let mut defs = String::new();
    let mut body = String::new();
    let mut id_counter = 0usize;

    // Backdrop matches the parent screen's fill (or a clean neutral) so translucent/borderless elements are readable.
    let backdrop = screen
        .node
        .fills
        .first()
        .map(String::as_str)
        .unwrap_or(if screen.theme == "dark" { "#18181B" } else { "#F4F4F5" });
    let _ = writeln!(body, r##"<rect x="0" y="0" width="{w:.1}" height="{h:.1}" fill="{}"/>"##, escape_xml(backdrop));

    emit_node_svg(elem, CROP_PADDING, CROP_PADDING, &mut defs, &mut body, &mut id_counter);

    let mut elements = Vec::new();
    collect_boxes(elem, 0.0, 0.0, &mut elements, 40);

    let svg = wrap_svg(w, h, &defs, &body);
    (
        svg,
        w,
        h,
        format!("`{}` inside `{}`", elem.id, screen.id),
        "element",
        screen.theme.clone(),
        elements,
    )
}

fn collect_boxes(node: &LaidOutNode, ox: f64, oy: f64, out: &mut Vec<ElementBox>, limit: usize) {
    if out.len() >= limit {
        return;
    }
    let ax = ox + node.x;
    let ay = oy + node.y;
    out.push(ElementBox {
        id: node.id.clone(),
        kind: node.kind.clone(),
        x: ax.round(),
        y: ay.round(),
        width: node.w.round(),
        height: node.h.round(),
    });
    for child in &node.children {
        collect_boxes(child, ax, ay, out, limit);
    }
}

fn wrap_svg(w: f64, h: f64, defs: &str, body: &str) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w:.1}" height="{h:.1}" viewBox="0 0 {w:.1} {h:.1}"><defs>{defs}</defs>{body}</svg>"#
    )
}

fn rasterize_svg(svg: &str, logical_w: f64, logical_h: f64) -> Result<(Vec<u8>, u32, u32), String> {
    let mut opt = usvg::Options {
        fontdb: fontdb(),
        font_family: "DejaVu Sans".to_string(),
        ..usvg::Options::default()
    };
    // Keep text crisp and predictable.
    opt.default_size = usvg::Size::from_wh(logical_w.max(1.0) as f32, logical_h.max(1.0) as f32)
        .unwrap_or_else(|| usvg::Size::from_wh(800.0, 600.0).unwrap());

    let tree = usvg::Tree::from_str(svg, &opt).map_err(|e| format!("failed to parse rendered SVG: {e}"))?;

    let max_dim = logical_w.max(logical_h).max(1.0);
    // Render small elements at 2x for readability, cap large artboards at MAX_RENDER_PX.
    let scale = if max_dim < 700.0 {
        (MAX_RENDER_PX / max_dim).min(2.0)
    } else if max_dim > MAX_RENDER_PX {
        MAX_RENDER_PX / max_dim
    } else {
        1.0
    };

    let px_w = (logical_w * scale).round().clamp(1.0, MAX_RENDER_PX) as u32;
    let px_h = (logical_h * scale).round().clamp(1.0, MAX_RENDER_PX) as u32;
    let mut pixmap = tiny_skia::Pixmap::new(px_w, px_h).ok_or("rendered image size was invalid")?;

    resvg::render(&tree, tiny_skia::Transform::from_scale(scale as f32, scale as f32), &mut pixmap.as_mut());
    let png = pixmap.encode_png().map_err(|e| format!("failed to encode PNG: {e}"))?;
    Ok((png, px_w, px_h))
}

// -----------------------------------------------------------------------------
// Document parsing, variable resolution, and component expansion
// -----------------------------------------------------------------------------

struct MergedPen {
    variables: BTreeMap<String, Value>,
    components: HashMap<String, Value>,
    children: Vec<Value>,
}

impl MergedPen {
    fn parse(files: &[(&str, &str)]) -> Result<Self, String> {
        let mut pen_files: Vec<(&str, &str)> = files.iter().copied().filter(|(p, _)| p.ends_with(".pen")).collect();
        if pen_files.is_empty() {
            return Err("this preview has no .pen files to screenshot (only OpenPencil .pen canvases can be screenshotted)".to_string());
        }
        pen_files.sort_by_key(|(p, _)| *p);

        let mut variables = BTreeMap::new();
        let mut children = Vec::new();
        for (path, content) in pen_files {
            let parsed: Value = serde_json::from_str(content).map_err(|e| format!("{path} is not valid JSON: {e}"))?;
            if let Some(vars) = parsed.get("variables").and_then(Value::as_object) {
                for (k, v) in vars {
                    variables.insert(k.clone(), v.clone());
                }
            }
            if let Some(list) = parsed.get("children").and_then(Value::as_array) {
                children.extend(list.iter().cloned());
            }
        }

        let mut components = HashMap::new();
        for child in &children {
            index_reusable(child, &mut components);
        }

        Ok(Self { variables, components, children })
    }
}

fn index_reusable(node: &Value, out: &mut HashMap<String, Value>) {
    if node.get("reusable").and_then(Value::as_bool) == Some(true) {
        if let Some(id) = node.get("id").and_then(Value::as_str) {
            out.insert(id.to_string(), node.clone());
        }
    }
    if let Some(children) = node.get("children").and_then(Value::as_array) {
        for child in children {
            index_reusable(child, out);
        }
    }
}

struct ResolveCtx<'a> {
    vars: &'a BTreeMap<String, Value>,
    components: &'a HashMap<String, Value>,
    theme: &'a str,
}

fn extract_theme(node: &Value) -> Option<String> {
    node.get("theme")
        .and_then(Value::as_object)
        .and_then(|obj| obj.get("theme").or_else(|| obj.values().next()))
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn expand_node(node: &Value, ctx: &ResolveCtx<'_>, depth: usize, diagnostics: &mut Vec<String>) -> Value {
    if depth >= MAX_REF_DEPTH {
        return node.clone();
    }
    if node_str(node, "type") == Some("ref") {
        let Some(ref_id) = node_str(node, "ref") else {
            return node.clone();
        };
        let Some(template) = ctx.components.get(ref_id) else {
            diagnostics.push(format!("Instance `{}` references unknown component `{ref_id}`.", node_str(node, "id").unwrap_or("?")));
            return node.clone();
        };
        let mut merged = template.clone();
        if let (Some(dst), Some(src)) = (merged.as_object_mut(), node.as_object()) {
            dst.remove("reusable");
            for (k, v) in src {
                if k != "type" && k != "ref" && k != "descendants" {
                    dst.insert(k.clone(), v.clone());
                }
            }
        }
        if let Some(descendants) = node.get("descendants").and_then(Value::as_object) {
            apply_descendants(&mut merged, descendants);
        }
        return expand_node(&merged, ctx, depth + 1, diagnostics);
    }

    let mut out = node.clone();
    if let Some(children) = out.get_mut("children").and_then(Value::as_array_mut) {
        for child in children.iter_mut() {
            *child = expand_node(child, ctx, depth + 1, diagnostics);
        }
    }
    out
}

fn apply_descendants(node: &mut Value, descendants: &serde_json::Map<String, Value>) {
    if let Some(id) = node.get("id").and_then(Value::as_str).map(str::to_string) {
        if let Some(overrides) = descendants.get(&id).and_then(Value::as_object) {
            if let Some(obj) = node.as_object_mut() {
                for (k, v) in overrides {
                    obj.insert(k.clone(), v.clone());
                }
            }
        }
    }
    if let Some(children) = node.get_mut("children").and_then(Value::as_array_mut) {
        for child in children.iter_mut() {
            apply_descendants(child, descendants);
        }
    }
}

fn lookup_var<'a>(name: &str, ctx: &'a ResolveCtx<'a>) -> Option<&'a Value> {
    let key = name.trim_start_matches('$').trim_start_matches("--");
    let def = ctx.vars.get(key).or_else(|| ctx.vars.get(&format!("--{key}")))?;
    let val = def.get("value").unwrap_or(def);
    if let Some(entries) = val.as_array() {
        // Pick matching theme entry, else fall back to first (light/default).
        for entry in entries {
            if let Some(t) = entry.get("theme").and_then(Value::as_object) {
                if t.values().any(|v| v.as_str() == Some(ctx.theme)) {
                    return entry.get("value");
                }
            }
        }
        return entries.first().and_then(|e| e.get("value").or(Some(e)));
    }
    Some(val)
}

fn resolve_color(val: Option<&Value>, ctx: &ResolveCtx<'_>, diagnostics: &mut Vec<String>) -> Option<String> {
    let raw = val?.as_str()?.trim();
    if raw.starts_with('$') {
        if let Some(resolved) = lookup_var(raw, ctx).and_then(Value::as_str) {
            return Some(resolved.to_string());
        }
        diagnostics.push(format!("Unresolved color token `{raw}`."));
        return None;
    }
    Some(raw.to_string())
}

fn resolve_fills(val: Option<&Value>, ctx: &ResolveCtx<'_>, diagnostics: &mut Vec<String>) -> Vec<String> {
    let Some(val) = val else { return Vec::new() };
    match val {
        Value::String(_) => resolve_color(Some(val), ctx, diagnostics).into_iter().collect(),
        Value::Array(items) => items.iter().filter_map(|item| resolve_color(Some(item), ctx, diagnostics)).collect(),
        Value::Object(obj) => resolve_color(obj.get("color").or_else(|| obj.get("fill")), ctx, diagnostics).into_iter().collect(),
        _ => Vec::new(),
    }
}

fn resolve_number(val: Option<&Value>, ctx: &ResolveCtx<'_>, diagnostics: &mut Vec<String>) -> Option<f64> {
    let val = val?;
    if let Some(n) = val.as_f64() {
        return Some(n);
    }
    if let Some(s) = val.as_str() {
        let s = s.trim();
        if s.starts_with('$') {
            if let Some(resolved) = lookup_var(s, ctx) {
                return resolved.as_f64().or_else(|| resolved.as_str().and_then(|v| v.parse::<f64>().ok()));
            }
            diagnostics.push(format!("Unresolved number token `{s}`."));
            return None;
        }
        if let Ok(n) = s.parse::<f64>() {
            return Some(n);
        }
    }
    None
}

fn resolve_string(val: Option<&Value>, ctx: &ResolveCtx<'_>) -> Option<String> {
    let s = val?.as_str()?.trim();
    if s.starts_with('$') {
        return lookup_var(s, ctx).and_then(Value::as_str).map(str::to_string);
    }
    Some(s.to_string())
}

fn node_str<'a>(node: &'a Value, key: &str) -> Option<&'a str> {
    node.get(key).and_then(Value::as_str)
}

// -----------------------------------------------------------------------------
// Layout engine
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
enum SizeSpec {
    Fixed(f64),
    Fill(Option<f64>),
    Fit(Option<f64>),
}

fn parse_size(val: Option<&Value>, ctx: &ResolveCtx<'_>, diagnostics: &mut Vec<String>) -> SizeSpec {
    let Some(val) = val else { return SizeSpec::Fit(None) };
    if let Some(n) = val.as_f64() {
        return SizeSpec::Fixed(n.max(0.0));
    }
    if let Some(s) = val.as_str() {
        let s = s.trim();
        if s.starts_with('$') {
            if let Some(n) = resolve_number(Some(val), ctx, diagnostics) {
                return SizeSpec::Fixed(n.max(0.0));
            }
            return SizeSpec::Fit(None);
        }
        if s == "fill_container" {
            return SizeSpec::Fill(None);
        }
        if let Some(inner) = s.strip_prefix("fill_container(").and_then(|r| r.strip_suffix(')')) {
            return SizeSpec::Fill(inner.trim().parse::<f64>().ok());
        }
        if s == "fit_content" {
            return SizeSpec::Fit(None);
        }
        if let Some(inner) = s.strip_prefix("fit_content(").and_then(|r| r.strip_suffix(')')) {
            return SizeSpec::Fit(inner.trim().parse::<f64>().ok());
        }
        if let Ok(n) = s.parse::<f64>() {
            return SizeSpec::Fixed(n.max(0.0));
        }
    }
    SizeSpec::Fit(None)
}

#[derive(Debug, Clone, Copy, Default)]
struct Padding {
    top: f64,
    right: f64,
    bottom: f64,
    left: f64,
}

fn parse_padding(val: Option<&Value>, ctx: &ResolveCtx<'_>, diagnostics: &mut Vec<String>) -> Padding {
    let Some(val) = val else { return Padding::default() };
    if let Some(n) = resolve_number(Some(val), ctx, diagnostics) {
        return Padding { top: n, right: n, bottom: n, left: n };
    }
    if let Some(arr) = val.as_array() {
        let nums: Vec<f64> = arr.iter().map(|v| resolve_number(Some(v), ctx, diagnostics).unwrap_or(0.0)).collect();
        return match nums.as_slice() {
            [a] => Padding { top: *a, right: *a, bottom: *a, left: *a },
            [v, h] => Padding { top: *v, right: *h, bottom: *v, left: *h },
            [t, r, b, l] => Padding { top: *t, right: *r, bottom: *b, left: *l },
            _ => Padding::default(),
        };
    }
    Padding::default()
}

fn parse_radii(val: Option<&Value>, ctx: &ResolveCtx<'_>, diagnostics: &mut Vec<String>) -> [f64; 4] {
    let Some(val) = val else { return [0.0; 4] };
    if let Some(n) = resolve_number(Some(val), ctx, diagnostics) {
        return [n.max(0.0); 4];
    }
    if let Some(arr) = val.as_array() {
        let nums: Vec<f64> = arr.iter().map(|v| resolve_number(Some(v), ctx, diagnostics).unwrap_or(0.0).max(0.0)).collect();
        if nums.len() == 4 {
            return [nums[0], nums[1], nums[2], nums[3]];
        }
    }
    [0.0; 4]
}

#[derive(Debug, Clone)]
struct StrokeSpec {
    color: String,
    top: f64,
    right: f64,
    bottom: f64,
    left: f64,
    uniform: bool,
    align: String,
    cap: Option<String>,
    join: Option<String>,
}

fn parse_stroke(val: Option<&Value>, ctx: &ResolveCtx<'_>, diagnostics: &mut Vec<String>) -> Option<StrokeSpec> {
    let obj = val?.as_object()?;
    let color = resolve_color(obj.get("fill").or_else(|| obj.get("color")), ctx, diagnostics)?;
    let align = obj.get("align").and_then(Value::as_str).unwrap_or("inside").to_string();
    let cap = obj.get("cap").and_then(Value::as_str).map(str::to_string);
    let join = obj.get("join").and_then(Value::as_str).map(str::to_string);
    let (top, right, bottom, left, uniform) = match obj.get("thickness") {
        Some(Value::Object(sides)) => (
            resolve_number(sides.get("top"), ctx, diagnostics).unwrap_or(0.0),
            resolve_number(sides.get("right"), ctx, diagnostics).unwrap_or(0.0),
            resolve_number(sides.get("bottom"), ctx, diagnostics).unwrap_or(0.0),
            resolve_number(sides.get("left"), ctx, diagnostics).unwrap_or(0.0),
            false,
        ),
        other => {
            let t = resolve_number(other, ctx, diagnostics).unwrap_or(1.0);
            (t, t, t, t, true)
        }
    };
    if top <= 0.0 && right <= 0.0 && bottom <= 0.0 && left <= 0.0 {
        return None;
    }
    Some(StrokeSpec { color, top, right, bottom, left, uniform, align, cap, join })
}

#[derive(Debug, Clone)]
struct ShadowSpec {
    color: String,
    dx: f64,
    dy: f64,
    blur: f64,
}

fn parse_shadows(val: Option<&Value>, ctx: &ResolveCtx<'_>, diagnostics: &mut Vec<String>) -> Vec<ShadowSpec> {
    let Some(val) = val else { return Vec::new() };
    let items: Vec<&Value> = match val {
        Value::Array(list) => list.iter().collect(),
        Value::Object(_) => vec![val],
        _ => return Vec::new(),
    };
    let mut out = Vec::new();
    for item in items {
        let Some(obj) = item.as_object() else { continue };
        if obj.get("type").and_then(Value::as_str) != Some("shadow") {
            continue;
        }
        if obj.get("shadowType").and_then(Value::as_str) == Some("inner") {
            continue;
        }
        let color = resolve_color(obj.get("color"), ctx, diagnostics).unwrap_or_else(|| "#00000026".to_string());
        let dx = obj.get("offset").and_then(|o| o.get("x")).and_then(Value::as_f64).unwrap_or(0.0);
        let dy = obj.get("offset").and_then(|o| o.get("y")).and_then(Value::as_f64).unwrap_or(4.0);
        let blur = obj.get("blur").and_then(Value::as_f64).unwrap_or(12.0);
        out.push(ShadowSpec { color, dx, dy, blur });
    }
    out
}

#[derive(Debug, Clone)]
struct TextData {
    lines: Vec<String>,
    font_family: String,
    font_size: f64,
    font_weight: u16,
    line_height_px: f64,
    letter_spacing: f64,
    align: String,
}

#[derive(Debug, Clone)]
struct LaidOutNode {
    id: String,
    kind: String,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    opacity: f64,
    rotation: f64,
    clip: bool,
    fills: Vec<String>,
    radii: [f64; 4],
    stroke: Option<StrokeSpec>,
    shadows: Vec<ShadowSpec>,
    text: Option<TextData>,
    geometry: Option<String>,
    children: Vec<LaidOutNode>,
}

struct LaidOutScreen {
    id: String,
    name: String,
    x: f64,
    y: f64,
    theme: String,
    node: LaidOutNode,
}

fn layout_node(
    raw: &Value,
    ctx: &ResolveCtx<'_>,
    avail_w: Option<f64>,
    avail_h: Option<f64>,
    diagnostics: &mut Vec<String>,
) -> LaidOutNode {
    let kind = node_str(raw, "type").unwrap_or("frame").to_string();
    let id = node_str(raw, "id").unwrap_or("node").to_string();
    let opacity = raw.get("opacity").and_then(Value::as_f64).unwrap_or(1.0).clamp(0.0, 1.0);
    let rotation = raw.get("rotation").and_then(Value::as_f64).unwrap_or(0.0);
    let clip = raw.get("clip").and_then(Value::as_bool).unwrap_or(false);
    let fills = resolve_fills(raw.get("fill"), ctx, diagnostics);
    let radii = parse_radii(raw.get("cornerRadius"), ctx, diagnostics);
    let stroke = parse_stroke(raw.get("stroke"), ctx, diagnostics);
    let shadows = parse_shadows(raw.get("effect"), ctx, diagnostics);

    if kind == "icon_font" {
        diagnostics.push(format!("Node `{id}` uses `icon_font`, which Studio doesn't render as an icon; draw a `path` node instead."));
    }

    let w_spec = parse_size(raw.get("width"), ctx, diagnostics);
    let h_spec = parse_size(raw.get("height"), ctx, diagnostics);

    match kind.as_str() {
        "text" | "icon_font" => {
            let content = node_str(raw, "content").or_else(|| node_str(raw, "iconFontName")).unwrap_or("");
            let font_family = map_font_family(resolve_string(raw.get("fontFamily"), ctx).as_deref());
            let font_size = resolve_number(raw.get("fontSize"), ctx, diagnostics).unwrap_or(16.0).max(6.0);
            let font_weight = parse_font_weight(raw.get("fontWeight"));
            let lh_raw = resolve_number(raw.get("lineHeight"), ctx, diagnostics).unwrap_or(1.35);
            let line_height_px = if lh_raw < 5.0 { font_size * lh_raw } else { lh_raw };
            let letter_spacing = resolve_number(raw.get("letterSpacing"), ctx, diagnostics).unwrap_or(0.0);
            let align = node_str(raw, "textAlign").unwrap_or("left").to_string();
            let fixed_width = node_str(raw, "textGrowth") == Some("fixed-width");

            let char_w = estimate_char_width(font_size, font_weight, &font_family) + letter_spacing.max(0.0) * 0.5;
            let constrained_w = match w_spec {
                SizeSpec::Fixed(n) => Some(n),
                SizeSpec::Fill(fb) => avail_w.or(fb),
                SizeSpec::Fit(fb) => if fixed_width { avail_w.or(fb) } else { None },
            };

            let lines = if let Some(max_w) = constrained_w {
                wrap_text(content, max_w, char_w)
            } else {
                content.lines().map(str::to_string).collect::<Vec<_>>()
            };
            let lines = if lines.is_empty() { vec![String::new()] } else { lines };

            let measured_w = lines
                .iter()
                .map(|line| (line.chars().count() as f64) * char_w)
                .fold(0.0_f64, f64::max)
                .ceil();
            let measured_h = (lines.len() as f64 * line_height_px).ceil();

            let w = match w_spec {
                SizeSpec::Fixed(n) => n,
                SizeSpec::Fill(fb) => avail_w.or(fb).unwrap_or(measured_w),
                SizeSpec::Fit(fb) => fb.unwrap_or(measured_w).max(if fixed_width { constrained_w.unwrap_or(0.0) } else { 0.0 }),
            };
            let h = match h_spec {
                SizeSpec::Fixed(n) => n,
                SizeSpec::Fill(fb) => avail_h.or(fb).unwrap_or(measured_h),
                SizeSpec::Fit(fb) => fb.unwrap_or(measured_h).max(measured_h),
            };

            LaidOutNode {
                id,
                kind: "text".into(),
                x: 0.0,
                y: 0.0,
                w,
                h,
                opacity,
                rotation,
                clip,
                fills,
                radii,
                stroke,
                shadows,
                text: Some(TextData {
                    lines,
                    font_family,
                    font_size,
                    font_weight,
                    line_height_px,
                    letter_spacing,
                    align,
                }),
                geometry: None,
                children: Vec::new(),
            }
        }
        "rectangle" | "ellipse" | "path" => {
            let w = match w_spec {
                SizeSpec::Fixed(n) => n,
                SizeSpec::Fill(fb) => avail_w.or(fb).unwrap_or(100.0),
                SizeSpec::Fit(fb) => fb.unwrap_or(24.0),
            };
            let h = match h_spec {
                SizeSpec::Fixed(n) => n,
                SizeSpec::Fill(fb) => avail_h.or(fb).unwrap_or(100.0),
                SizeSpec::Fit(fb) => fb.unwrap_or(24.0),
            };
            let geometry = node_str(raw, "geometry").map(str::to_string);
            LaidOutNode {
                id,
                kind,
                x: 0.0,
                y: 0.0,
                w,
                h,
                opacity,
                rotation,
                clip,
                fills,
                radii,
                stroke,
                shadows,
                text: None,
                geometry,
                children: Vec::new(),
            }
        }
        _ => {
            // Frame (default layout is "horizontal" per OpenPencil spec when omitted).
            let layout = node_str(raw, "layout").unwrap_or("horizontal");
            let pad = parse_padding(raw.get("padding"), ctx, diagnostics);
            let gap = resolve_number(raw.get("gap"), ctx, diagnostics).unwrap_or(0.0).max(0.0);
            let justify = node_str(raw, "justifyContent").unwrap_or("start");
            let align = node_str(raw, "alignItems").unwrap_or("start");

            let known_w = match w_spec {
                SizeSpec::Fixed(n) => Some(n),
                SizeSpec::Fill(fb) => avail_w.or(fb),
                SizeSpec::Fit(_) => None,
            };
            let known_h = match h_spec {
                SizeSpec::Fixed(n) => Some(n),
                SizeSpec::Fill(fb) => avail_h.or(fb),
                SizeSpec::Fit(_) => None,
            };

            let inner_w = known_w.map(|w| (w - pad.left - pad.right).max(0.0));
            let inner_h = known_h.map(|h| (h - pad.top - pad.bottom).max(0.0));

            let raw_children: Vec<&Value> = raw
                .get("children")
                .and_then(Value::as_array)
                .map(|arr| {
                    arr.iter()
                        .filter(|c| c.get("enabled").and_then(Value::as_bool) != Some(false))
                        .collect()
                })
                .unwrap_or_default();

            let mut children = Vec::with_capacity(raw_children.len());

            if layout == "none" {
                let mut content_w = 0.0_f64;
                let mut content_h = 0.0_f64;
                for rc in &raw_children {
                    let cx = resolve_number(rc.get("x"), ctx, diagnostics).unwrap_or(0.0);
                    let cy = resolve_number(rc.get("y"), ctx, diagnostics).unwrap_or(0.0);
                    let mut child = layout_node(rc, ctx, inner_w, inner_h, diagnostics);
                    child.x = cx;
                    child.y = cy;
                    content_w = content_w.max(cx + child.w);
                    content_h = content_h.max(cy + child.h);
                    children.push(child);
                }
                let w = known_w.unwrap_or_else(|| match w_spec {
                    SizeSpec::Fit(Some(fb)) => fb.max(content_w + pad.left + pad.right),
                    _ => content_w + pad.left + pad.right,
                });
                let h = known_h.unwrap_or_else(|| match h_spec {
                    SizeSpec::Fit(Some(fb)) => fb.max(content_h + pad.top + pad.bottom),
                    _ => content_h + pad.top + pad.bottom,
                });
                return LaidOutNode {
                    id,
                    kind: "frame".into(),
                    x: 0.0,
                    y: 0.0,
                    w,
                    h,
                    opacity,
                    rotation,
                    clip,
                    fills,
                    radii,
                    stroke,
                    shadows,
                    text: None,
                    geometry: None,
                    children,
                };
            }

            let is_vertical = layout == "vertical";

            // First pass: measure non-fill main-axis children to compute remaining space for fill_container children.
            let mut fill_count = 0usize;
            let mut fixed_main = 0.0_f64;
            let mut first_pass: Vec<Option<LaidOutNode>> = Vec::with_capacity(raw_children.len());

            for rc in &raw_children {
                let main_spec = if is_vertical {
                    parse_size(rc.get("height"), ctx, diagnostics)
                } else {
                    parse_size(rc.get("width"), ctx, diagnostics)
                };
                if matches!(main_spec, SizeSpec::Fill(_)) {
                    fill_count += 1;
                    first_pass.push(None);
                } else {
                    let child = if is_vertical {
                        layout_node(rc, ctx, inner_w, None, diagnostics)
                    } else {
                        layout_node(rc, ctx, None, inner_h, diagnostics)
                    };
                    fixed_main += if is_vertical { child.h } else { child.w };
                    first_pass.push(Some(child));
                }
            }

            let total_gaps = if raw_children.len() > 1 {
                gap * (raw_children.len() - 1) as f64
            } else {
                0.0
            };
            let fill_share = if fill_count > 0 {
                let avail_main = if is_vertical { inner_h } else { inner_w };
                avail_main.map(|m| ((m - fixed_main - total_gaps) / fill_count as f64).max(0.0))
            } else {
                None
            };

            for (rc, slot) in raw_children.iter().zip(first_pass.into_iter()) {
                let child = match slot {
                    Some(c) => c,
                    None => {
                        if is_vertical {
                            layout_node(rc, ctx, inner_w, fill_share, diagnostics)
                        } else {
                            layout_node(rc, ctx, fill_share, inner_h, diagnostics)
                        }
                    }
                };
                children.push(child);
            }

            let sum_main: f64 = children.iter().map(|c| if is_vertical { c.h } else { c.w }).sum::<f64>() + total_gaps;
            let max_cross: f64 = children.iter().map(|c| if is_vertical { c.w } else { c.h }).fold(0.0_f64, f64::max);

            let w = known_w.unwrap_or_else(|| {
                let natural = (if is_vertical { max_cross } else { sum_main }) + pad.left + pad.right;
                match w_spec {
                    SizeSpec::Fit(Some(fb)) => fb.max(natural),
                    _ => natural,
                }
            });
            let h = known_h.unwrap_or_else(|| {
                let natural = (if is_vertical { sum_main } else { max_cross }) + pad.top + pad.bottom;
                match h_spec {
                    SizeSpec::Fit(Some(fb)) => fb.max(natural),
                    _ => natural,
                }
            });

            let final_inner_w = (w - pad.left - pad.right).max(0.0);
            let final_inner_h = (h - pad.top - pad.bottom).max(0.0);
            let final_main = if is_vertical { final_inner_h } else { final_inner_w };
            let final_cross = if is_vertical { final_inner_w } else { final_inner_h };

            let (mut cursor, step_gap) = match justify {
                "center" => (((final_main - sum_main).max(0.0)) / 2.0, gap),
                "end" => ((final_main - sum_main).max(0.0), gap),
                "space-between" | "space_between" if children.len() > 1 => {
                    let raw_sum: f64 = children.iter().map(|c| if is_vertical { c.h } else { c.w }).sum();
                    (0.0, ((final_main - raw_sum).max(0.0)) / (children.len() - 1) as f64)
                }
                _ => (0.0, gap),
            };

            for child in &mut children {
                let child_cross = if is_vertical { child.w } else { child.h };
                let cross_pos = match align {
                    "center" => (final_cross - child_cross) / 2.0,
                    "end" => final_cross - child_cross,
                    "stretch" => {
                        if is_vertical {
                            child.w = final_cross;
                        } else {
                            child.h = final_cross;
                        }
                        0.0
                    }
                    _ => 0.0,
                };
                if is_vertical {
                    child.x = pad.left + cross_pos;
                    child.y = pad.top + cursor;
                    cursor += child.h + step_gap;
                } else {
                    child.x = pad.left + cursor;
                    child.y = pad.top + cross_pos;
                    cursor += child.w + step_gap;
                }

                if child.w <= 0.0 || child.h <= 0.0 {
                    diagnostics.push(format!("Node `{}` has collapsed size ({:.0}x{:.0}).", child.id, child.w, child.h));
                } else if !clip && (child.x + child.w > w + 2.0 || child.y + child.h > h + 2.0) {
                    diagnostics.push(format!(
                        "Node `{}` ({:.0}x{:.0} at {:.0},{:.0}) overflows parent `{}` ({:.0}x{:.0}).",
                        child.id, child.w, child.h, child.x, child.y, id, w, h
                    ));
                }
            }

            LaidOutNode {
                id,
                kind: "frame".into(),
                x: 0.0,
                y: 0.0,
                w,
                h,
                opacity,
                rotation,
                clip,
                fills,
                radii,
                stroke,
                shadows,
                text: None,
                geometry: None,
                children,
            }
        }
    }
}

// -----------------------------------------------------------------------------
// SVG emission
// -----------------------------------------------------------------------------

fn emit_node_svg(
    node: &LaidOutNode,
    ox: f64,
    oy: f64,
    defs: &mut String,
    body: &mut String,
    id_counter: &mut usize,
) {
    let mut group_attrs = String::new();
    if (node.opacity - 1.0).abs() > 0.001 {
        let _ = write!(group_attrs, r#" opacity="{:.3}""#, node.opacity);
    }
    if node.rotation.abs() > 0.01 {
        let cx = ox + node.w / 2.0;
        let cy = oy + node.h / 2.0;
        let _ = write!(group_attrs, r#" transform="rotate({:.2} {cx:.1} {cy:.1})""#, node.rotation);
    }
    if !node.shadows.is_empty() {
        *id_counter += 1;
        let fid = format!("sh{}", *id_counter);
        let s = &node.shadows[0];
        let (hex, alpha) = split_hex_alpha(&s.color);
        let std_dev = (s.blur / 2.0).max(0.5);
        let _ = writeln!(
            defs,
            r#"<filter id="{fid}" x="-30%" y="-30%" width="160%" height="160%"><feDropShadow dx="{:.1}" dy="{:.1}" stdDeviation="{std_dev:.1}" flood-color="{hex}" flood-opacity="{alpha:.3}"/></filter>"#,
            s.dx, s.dy
        );
        let _ = write!(group_attrs, r#" filter="url(#{fid})""#);
    }

    let opened_group = !group_attrs.is_empty();
    if opened_group {
        let _ = writeln!(body, "<g{group_attrs}>");
    }

    match node.kind.as_str() {
        "frame" | "rectangle" => {
            let shape_path = rounded_rect_path(ox, oy, node.w, node.h, node.radii);
            for fill in &node.fills {
                let (hex, alpha) = split_hex_alpha(fill);
                let _ = writeln!(
                    body,
                    r#"<path d="{shape_path}" fill="{hex}" fill-opacity="{alpha:.3}"/>"#
                );
            }
            if let Some(stroke) = &node.stroke {
                emit_stroke_svg(body, ox, oy, node.w, node.h, node.radii, stroke);
            }
            if !node.children.is_empty() {
                if node.clip {
                    *id_counter += 1;
                    let cid = format!("cp{}", *id_counter);
                    let _ = writeln!(defs, r#"<clipPath id="{cid}"><path d="{shape_path}"/></clipPath>"#);
                    let _ = writeln!(body, r#"<g clip-path="url(#{cid})">"#);
                }
                for child in &node.children {
                    emit_node_svg(child, ox + child.x, oy + child.y, defs, body, id_counter);
                }
                if node.clip {
                    let _ = writeln!(body, "</g>");
                }
            }
        }
        "ellipse" => {
            let cx = ox + node.w / 2.0;
            let cy = oy + node.h / 2.0;
            let rx = (node.w / 2.0).max(0.5);
            let ry = (node.h / 2.0).max(0.5);
            let (fill_hex, fill_alpha) = node.fills.last().map(|f| split_hex_alpha(f)).unwrap_or(("none".into(), 0.0));
            let stroke_attr = match &node.stroke {
                Some(s) => {
                    let (sh, sa) = split_hex_alpha(&s.color);
                    format!(r#" stroke="{sh}" stroke-opacity="{sa:.3}" stroke-width="{:.2}""#, s.top)
                }
                None => String::new(),
            };
            let _ = writeln!(
                body,
                r#"<ellipse cx="{cx:.1}" cy="{cy:.1}" rx="{rx:.1}" ry="{ry:.1}" fill="{fill_hex}" fill-opacity="{fill_alpha:.3}"{stroke_attr}/>"#
            );
        }
        "path" => {
            if let Some(d) = &node.geometry {
                let fill_attr = match node.fills.last() {
                    Some(f) => {
                        let (hex, alpha) = split_hex_alpha(f);
                        format!(r#"fill="{hex}" fill-opacity="{alpha:.3}""#)
                    }
                    None => r#"fill="none""#.to_string(),
                };
                let stroke_attr = match &node.stroke {
                    Some(s) => {
                        let (sh, sa) = split_hex_alpha(&s.color);
                        let cap = s.cap.as_deref().unwrap_or("round");
                        let join = s.join.as_deref().unwrap_or("round");
                        format!(
                            r#" stroke="{sh}" stroke-opacity="{sa:.3}" stroke-width="{:.2}" stroke-linecap="{cap}" stroke-linejoin="{join}""#,
                            s.top
                        )
                    }
                    None => String::new(),
                };
                // OpenPencil SVG paths are typically authored in a 24x24 icon box; translate to (ox, oy) and scale if needed.
                let scale_x = (node.w / 24.0).max(0.1);
                let scale_y = (node.h / 24.0).max(0.1);
                let _ = writeln!(
                    body,
                    r#"<g transform="translate({ox:.1},{oy:.1}) scale({scale_x:.3},{scale_y:.3})"><path d="{}" {fill_attr}{stroke_attr}/></g>"#,
                    escape_xml(d)
                );
            }
        }
        "text" => {
            if let Some(td) = &node.text {
                let (hex, alpha) = node.fills.last().map(|f| split_hex_alpha(f)).unwrap_or(("#111111".into(), 1.0));
                let (anchor, tx) = match td.align.as_str() {
                    "center" => ("middle", ox + node.w / 2.0),
                    "right" => ("end", ox + node.w),
                    _ => ("start", ox),
                };
                let ls_attr = if td.letter_spacing.abs() > 0.01 {
                    format!(r#" letter-spacing="{:.2}""#, td.letter_spacing)
                } else {
                    String::new()
                };
                let baseline_offset = td.font_size * 0.85;
                for (i, line) in td.lines.iter().enumerate() {
                    let ty = oy + baseline_offset + (i as f64) * td.line_height_px;
                    let _ = writeln!(
                        body,
                        r#"<text x="{tx:.1}" y="{ty:.1}" font-family="{}" font-size="{:.1}" font-weight="{}" text-anchor="{anchor}" fill="{hex}" fill-opacity="{alpha:.3}"{ls_attr}>{}</text>"#,
                        escape_xml(&td.font_family),
                        td.font_size,
                        td.font_weight,
                        escape_xml(line)
                    );
                }
            }
        }
        _ => {}
    }

    if opened_group {
        let _ = writeln!(body, "</g>");
    }
}

fn emit_stroke_svg(body: &mut String, x: f64, y: f64, w: f64, h: f64, radii: [f64; 4], stroke: &StrokeSpec) {
    let (hex, alpha) = split_hex_alpha(&stroke.color);
    if stroke.uniform {
        let inset = match stroke.align.as_str() {
            "inside" => stroke.top / 2.0,
            "outside" => -stroke.top / 2.0,
            _ => 0.0,
        };
        let sx = x + inset;
        let sy = y + inset;
        let sw = (w - inset * 2.0).max(0.0);
        let sh = (h - inset * 2.0).max(0.0);
        let sr = radii.map(|r| (r - inset).max(0.0));
        let path = rounded_rect_path(sx, sy, sw, sh, sr);
        let _ = writeln!(
            body,
            r#"<path d="{path}" fill="none" stroke="{hex}" stroke-opacity="{alpha:.3}" stroke-width="{:.2}"/>"#,
            stroke.top
        );
    } else {
        if stroke.top > 0.0 {
            let _ = writeln!(body, r#"<line x1="{x:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="{hex}" stroke-opacity="{alpha:.3}" stroke-width="{:.2}"/>"#, y + stroke.top / 2.0, x + w, y + stroke.top / 2.0, stroke.top);
        }
        if stroke.bottom > 0.0 {
            let _ = writeln!(body, r#"<line x1="{x:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="{hex}" stroke-opacity="{alpha:.3}" stroke-width="{:.2}"/>"#, y + h - stroke.bottom / 2.0, x + w, y + h - stroke.bottom / 2.0, stroke.bottom);
        }
        if stroke.left > 0.0 {
            let _ = writeln!(body, r#"<line x1="{:.1}" y1="{y:.1}" x2="{:.1}" y2="{:.1}" stroke="{hex}" stroke-opacity="{alpha:.3}" stroke-width="{:.2}"/>"#, x + stroke.left / 2.0, x + stroke.left / 2.0, y + h, stroke.left);
        }
        if stroke.right > 0.0 {
            let _ = writeln!(body, r#"<line x1="{:.1}" y1="{y:.1}" x2="{:.1}" y2="{:.1}" stroke="{hex}" stroke-opacity="{alpha:.3}" stroke-width="{:.2}"/>"#, x + w - stroke.right / 2.0, x + w - stroke.right / 2.0, y + h, stroke.right);
        }
    }
}

fn rounded_rect_path(x: f64, y: f64, w: f64, h: f64, radii: [f64; 4]) -> String {
    let max_r = (w / 2.0).min(h / 2.0).max(0.0);
    let [tl, tr, br, bl] = radii.map(|r| r.clamp(0.0, max_r));
    format!(
        "M {:.1} {:.1} H {:.1} Q {:.1} {:.1} {:.1} {:.1} V {:.1} Q {:.1} {:.1} {:.1} {:.1} H {:.1} Q {:.1} {:.1} {:.1} {:.1} V {:.1} Q {:.1} {:.1} {:.1} {:.1} Z",
        x + tl, y,
        x + w - tr, x + w, y, x + w, y + tr,
        y + h - br, x + w, y + h, x + w - br, y + h,
        x + bl, x, y + h, x, y + h - bl,
        y + tl, x, y, x + tl, y
    )
}

// -----------------------------------------------------------------------------
// Text & color utilities
// -----------------------------------------------------------------------------

fn map_font_family(name: Option<&str>) -> String {
    let Some(raw) = name.map(str::trim).filter(|s| !s.is_empty()) else {
        return "DejaVu Sans, sans-serif".to_string();
    };
    let lower = raw.to_ascii_lowercase();
    if lower.contains("mono") || lower.contains("code") || lower.contains("jetbrains") || lower.contains("fira") {
        format!("{raw}, DejaVu Sans Mono, monospace")
    } else if lower.contains("serif") || lower.contains("fraunces") || lower.contains("playfair") || lower.contains("georgia") || lower.contains("lora") || lower.contains("merriweather") {
        format!("{raw}, DejaVu Serif, serif")
    } else {
        format!("{raw}, DejaVu Sans, sans-serif")
    }
}

fn parse_font_weight(val: Option<&Value>) -> u16 {
    match val {
        Some(Value::Number(n)) => n.as_u64().unwrap_or(400).clamp(100, 900) as u16,
        Some(Value::String(s)) => match s.trim().to_ascii_lowercase().as_str() {
            "thin" => 100,
            "extralight" => 200,
            "light" => 300,
            "normal" | "regular" => 400,
            "medium" => 500,
            "semibold" | "semi-bold" => 600,
            "bold" => 700,
            "extrabold" | "extra-bold" => 800,
            "black" => 900,
            other => other.parse::<u16>().unwrap_or(400).clamp(100, 900),
        },
        _ => 400,
    }
}

fn estimate_char_width(font_size: f64, font_weight: u16, font_family: &str) -> f64 {
    let base = if font_family.contains("monospace") {
        0.60
    } else if font_weight >= 600 {
        0.58
    } else {
        0.55
    };
    font_size * base
}

fn wrap_text(text: &str, max_width: f64, char_w: f64) -> Vec<String> {
    let max_chars = (max_width / char_w.max(1.0)).floor().max(1.0) as usize;
    let mut lines = Vec::new();
    for para in text.split('\n') {
        if para.is_empty() {
            lines.push(String::new());
            continue;
        }
        let mut current = String::new();
        for word in para.split_whitespace() {
            let word_len = word.chars().count();
            if current.is_empty() {
                current.push_str(word);
            } else if current.chars().count() + 1 + word_len <= max_chars {
                current.push(' ');
                current.push_str(word);
            } else {
                lines.push(current);
                current = word.to_string();
            }
        }
        if !current.is_empty() {
            lines.push(current);
        }
    }
    lines
}

fn split_hex_alpha(color: &str) -> (String, f64) {
    let c = color.trim();
    if let Some(hex) = c.strip_prefix('#') {
        if hex.len() == 8 {
            if let Ok(a) = u8::from_str_radix(&hex[6..8], 16) {
                return (format!("#{}", &hex[0..6]), f64::from(a) / 255.0);
            }
        } else if hex.len() == 4 {
            let chars: Vec<char> = hex.chars().collect();
            if let Ok(a) = u8::from_str_radix(&format!("{}{}", chars[3], chars[3]), 16) {
                return (format!("#{}{}{}", chars[0], chars[1], chars[2]), f64::from(a) / 255.0);
            }
        }
    }
    (c.to_string(), 1.0)
}

fn rects_overlap(ax: f64, ay: f64, aw: f64, ah: f64, bx: f64, by: f64, bw: f64, bh: f64) -> bool {
    ax < bx + bw && ax + aw > bx && ay < by + bh && ay + ah > by
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_pen() -> String {
        json!({
            "version": "1.0",
            "themes": { "theme": ["light", "dark"] },
            "variables": {
                "color-background": { "type": "color", "value": [{ "value": "#F7F4EE" }, { "value": "#141311", "theme": { "theme": "dark" } }] },
                "color-foreground": { "type": "color", "value": [{ "value": "#1C1917" }, { "value": "#F5F5F4", "theme": { "theme": "dark" } }] },
                "color-primary": { "type": "color", "value": "#1F6F5C" },
                "color-card": { "type": "color", "value": "#FFFFFF" },
                "color-border": { "type": "color", "value": "#E5E0D8" },
                "radius-md": { "type": "number", "value": 12 },
                "font-heading": { "type": "string", "value": "Fraunces" },
                "font-body": { "type": "string", "value": "Instrument Sans" }
            },
            "children": [
                {
                    "type": "frame",
                    "id": "design-system",
                    "name": "Design System",
                    "x": 0,
                    "y": 0,
                    "width": 400,
                    "height": "fit_content",
                    "layout": "vertical",
                    "gap": 16,
                    "padding": 24,
                    "fill": "$color-background",
                    "theme": { "theme": "light" },
                    "children": [
                        {
                            "type": "frame",
                            "id": "button-primary",
                            "reusable": true,
                            "layout": "horizontal",
                            "padding": [12, 20],
                            "fill": "$color-primary",
                            "cornerRadius": "$radius-md",
                            "children": [
                                { "type": "text", "id": "button-label", "content": "Order now", "fontFamily": "$font-body", "fontSize": 15, "fontWeight": 600, "fill": "#FFFFFF" }
                            ]
                        }
                    ]
                },
                {
                    "type": "frame",
                    "id": "menu-phone",
                    "name": "Menu — Phone",
                    "x": 520,
                    "y": 0,
                    "width": 390,
                    "height": "fit_content",
                    "layout": "vertical",
                    "gap": 20,
                    "padding": [24, 20],
                    "fill": "$color-background",
                    "theme": { "theme": "light" },
                    "children": [
                        { "type": "text", "id": "menu-title", "content": "Roti hari ini", "fontFamily": "$font-heading", "fontSize": 28, "fontWeight": 600, "fill": "$color-foreground" },
                        {
                            "type": "frame",
                            "id": "menu-card",
                            "width": "fill_container",
                            "layout": "horizontal",
                            "gap": 12,
                            "padding": 12,
                            "fill": "$color-card",
                            "cornerRadius": "$radius-md",
                            "stroke": { "align": "inside", "thickness": 1, "fill": "$color-border" },
                            "children": [
                                { "type": "rectangle", "id": "menu-photo", "width": 64, "height": 64, "cornerRadius": 8, "fill": "#D9C7A7" },
                                { "type": "ref", "id": "menu-cta", "ref": "button-primary", "descendants": { "button-label": { "content": "Add" } } }
                            ]
                        }
                    ]
                }
            ]
        })
        .to_string()
    }

    #[test]
    fn renders_overview_screen_and_individual_element_to_valid_png() {
        let pen = sample_pen();
        let files = [("canvas.pen", pen.as_str())];

        let overview = render_pen_screenshot(&files, None, None).unwrap();
        assert_eq!(overview.target_kind, "overview");
        assert_eq!(overview.screens.len(), 2);
        let bytes = base64::engine::general_purpose::STANDARD.decode(&overview.png_base64).unwrap();
        assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"));

        let screen = render_pen_screenshot(&files, Some("menu-phone"), Some("dark")).unwrap();
        assert_eq!(screen.target_kind, "screen");
        assert_eq!(screen.theme, "dark");
        assert!(screen.width > 0 && screen.height > 0);

        let element = render_pen_screenshot(&files, Some("menu-card"), None).unwrap();
        assert_eq!(element.target_kind, "element");
        assert!(element.elements.iter().any(|e| e.id == "menu-cta"));
    }

    #[test]
    fn reports_diagnostics_for_overlapping_screens_and_unknown_tokens() {
        let pen = json!({
            "version": "1.0",
            "children": [
                { "type": "frame", "id": "a", "x": 0, "y": 0, "width": 300, "height": 300, "fill": "$missing-token" },
                { "type": "frame", "id": "b", "x": 100, "y": 0, "width": 300, "height": 300 }
            ]
        })
        .to_string();
        let shot = render_pen_screenshot(&[("canvas.pen", pen.as_str())], None, None).unwrap();
        assert!(shot.diagnostics.iter().any(|d| d.contains("Unresolved color token")));
        assert!(shot.diagnostics.iter().any(|d| d.contains("overlap")));
        assert!(shot.diagnostics.iter().any(|d| d.contains("has no `fill`")));
    }
}
