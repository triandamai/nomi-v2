//! Nomi's look in email: each agent's shape and colours, the same as in the app
//! (frontend/src/lib/components/m3/shapes.ts — keep the two in sync). Shapes are drawn here and
//! rasterized to PNG, since most mail apps show neither SVG nor CSS gradients on shapes.

/// One agent's look: which shape it wears and in which gradient tone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Look {
    pub shape: &'static str,
    pub tone: &'static str,
}

pub const NOMI_LOOK: Look = Look { shape: "cookie9", tone: "glow" };

/// (name, lobes, amplitude, phase): the polar outline r(θ) = R·(1 + amplitude·cos(lobes·(θ − phase))).
const SHAPES: &[(&str, f64, f64, f64)] = &[
    ("cookie9", 9.0, 0.09, 0.0),
    ("sunny8", 8.0, 0.06, 0.0),
    ("cookie6", 6.0, 0.1, 0.0),
    ("clover4", 4.0, 0.2, 0.0),
    ("flower5", 5.0, 0.16, 0.0),
    ("circle", 1.0, 0.0, 0.0),
    ("sunny12", 12.0, 0.045, 0.0),
    ("clover3", 3.0, 0.19, -std::f64::consts::FRAC_PI_2),
    ("puffy7", 7.0, 0.13, 0.0),
    ("burst16", 16.0, 0.055, 0.0),
    ("wave10", 10.0, 0.08, 0.0),
    ("soft-square", 4.0, 0.075, std::f64::consts::FRAC_PI_4),
    ("soft-triangle", 3.0, 0.12, -std::f64::consts::FRAC_PI_2),
    ("pentagon", 5.0, 0.055, -std::f64::consts::FRAC_PI_2),
    ("pill", 2.0, 0.17, 0.0),
];

/// A gradient tone: its stops, a deep accent for text and small marks, and the ink that reads on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tone {
    pub name: &'static str,
    pub stops: &'static [&'static str],
    pub accent: &'static str,
    pub ink: &'static str,
}

const TONES: &[Tone] = &[
    Tone { name: "glow", stops: &["#e4f76a", "#5be08f", "#1db9a0"], accent: "#0b6b4a", ink: "#062019" },
    Tone { name: "ember", stops: &["#ffd27a", "#ff7a4a"], accent: "#b8430f", ink: "#2b0d00" },
    Tone { name: "tide", stops: &["#c4f3ea", "#3fb8c8"], accent: "#0e7490", ink: "#002a30" },
    Tone { name: "sky", stops: &["#d6ecff", "#7ab0ff"], accent: "#3e7be0", ink: "#0a2340" },
    Tone { name: "bloom", stops: &["#ffd9e2", "#ff8fa8"], accent: "#be185d", ink: "#3d0716" },
    Tone { name: "dusk", stops: &["#e6dcff", "#9b7bff"], accent: "#6d4fd8", ink: "#1c0b4a" },
    Tone { name: "citrus", stops: &["#fff4a3", "#ffc23c"], accent: "#a16207", ink: "#2e2100" },
    Tone { name: "slate", stops: &["#dde6ec", "#7d93a3"], accent: "#475569", ink: "#0f1d26" },
];

/// The face's ink: the same two dots on every crew member.
const FACE_INK: &str = "#062019";

pub fn tone(name: &str) -> Tone {
    TONES.iter().copied().find(|t| t.name == name).unwrap_or(TONES[0])
}

fn shape_spec(name: &str) -> (f64, f64, f64) {
    SHAPES.iter().find(|(n, ..)| *n == name).map(|(_, l, a, p)| (*l, *a, *p)).unwrap_or((9.0, 0.09, 0.0))
}

pub fn is_shape(name: &str) -> bool {
    SHAPES.iter().any(|(n, ..)| *n == name)
}

pub fn is_tone(name: &str) -> bool {
    TONES.iter().any(|t| t.name == name)
}

/// Which look a built-in agent wears, by agent_type or display name. Anything unknown is Nomi.
pub fn agent_look(agent: &str) -> Look {
    let key = agent.to_lowercase();
    let look = |shape, tone| Look { shape, tone };
    // The crew's names (nomi_agent_core::crew) wear their agent's look.
    let key = match key.as_str() {
        "finley" => "money".to_string(),
        "cadence" => "reminders".to_string(),
        "paige" => "files".to_string(),
        "miles" => "planning".to_string(),
        "ada" => "coding".to_string(),
        "sloane" => "workspace".to_string(),
        _ => key,
    };
    if key.contains("money") || key.contains("budget") {
        look("sunny8", "ember")
    } else if key.contains("cod") {
        look("cookie6", "tide")
    } else if key.contains("plan") {
        look("clover4", "sky")
    } else if key.contains("personality") || key.contains("memory") {
        look("flower5", "bloom")
    } else if key.contains("supervisor") {
        look("sunny12", "dusk")
    } else if key.contains("reminder") {
        look("clover3", "citrus")
    } else if key == "files" {
        look("soft-square", "slate")
    } else if key == "workspace" {
        look("puffy7", "sky")
    } else {
        NOMI_LOOK
    }
}

/// The outline in the 48×48 box, as an SVG path.
pub fn shape_path(shape: &str) -> String {
    const POINTS: usize = 120;
    const RADIUS: f64 = 20.0;
    const CENTER: f64 = 24.0;
    let (lobes, amplitude, phase) = shape_spec(shape);
    let mut path = String::with_capacity(POINTS * 14);
    for i in 0..POINTS {
        let theta = 2.0 * std::f64::consts::PI * i as f64 / POINTS as f64;
        let r = RADIUS * (1.0 + amplitude * (lobes * (theta - phase)).cos());
        path.push(if i == 0 { 'M' } else { 'L' });
        path.push_str(&format!("{:.2} {:.2} ", CENTER + r * theta.cos(), CENTER + r * theta.sin()));
    }
    path.push('Z');
    path
}

/// The shape as an SVG, with the two-dot face when `face`.
pub fn shape_svg(shape: &str, tone_name: &str, face: bool) -> String {
    let t = tone(tone_name);
    let last = t.stops.len().saturating_sub(1).max(1) as f64;
    let stops: String = t
        .stops
        .iter()
        .enumerate()
        .map(|(i, c)| format!(r#"<stop offset="{}" stop-color="{c}"/>"#, i as f64 / last))
        .collect();
    let face = if face {
        format!(r#"<circle cx="19" cy="22" r="2.6" fill="{FACE_INK}"/><circle cx="29" cy="22" r="2.6" fill="{FACE_INK}"/>"#)
    } else {
        String::new()
    };
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 48 48"><defs><linearGradient id="g" x1="0" y1="0" x2="1" y2="1">{stops}</linearGradient></defs><path d="{}" fill="url(#g)"/>{face}</svg>"#,
        shape_path(shape)
    )
}

/// The shape as a `px`×`px` PNG with a transparent background.
pub fn shape_png(shape: &str, tone_name: &str, face: bool, px: u32) -> Vec<u8> {
    use resvg::{tiny_skia, usvg};
    let svg = shape_svg(shape, tone_name, face);
    let tree = usvg::Tree::from_str(&svg, &usvg::Options::default()).expect("the shape SVG is well-formed");
    let mut pixmap = tiny_skia::Pixmap::new(px, px).expect("a non-zero size");
    let scale = px as f32 / 48.0;
    resvg::render(&tree, tiny_skia::Transform::from_scale(scale, scale), &mut pixmap.as_mut());
    pixmap.encode_png().expect("PNG encoding of an in-memory pixmap")
}

/// `color` mixed toward white by `amount` (0 = the colour, 1 = white), as `#rrggbb`.
pub fn lighten(color: &str, amount: f64) -> String {
    let hex = color.trim_start_matches('#');
    let channel = |i: usize| u8::from_str_radix(hex.get(i..i + 2).unwrap_or("ff"), 16).unwrap_or(255) as f64;
    let mix = |c: f64| (c + (255.0 - c) * amount.clamp(0.0, 1.0)).round() as u8;
    format!("#{:02x}{:02x}{:02x}", mix(channel(0)), mix(channel(2)), mix(channel(4)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_agents_wear_their_app_looks() {
        assert_eq!(agent_look("chitchat"), NOMI_LOOK);
        assert_eq!(agent_look("Money"), Look { shape: "sunny8", tone: "ember" });
        assert_eq!(agent_look("Finley"), Look { shape: "sunny8", tone: "ember" });
        assert_eq!(agent_look("Paige"), agent_look("files"));
        assert_eq!(agent_look("reminders"), Look { shape: "clover3", tone: "citrus" });
        assert_eq!(agent_look("workspace"), Look { shape: "puffy7", tone: "sky" });
    }

    #[test]
    fn every_shape_renders_to_a_png() {
        for (name, ..) in SHAPES {
            let png = shape_png(name, "glow", true, 96);
            assert_eq!(&png[1..4], b"PNG", "{name}");
            assert!(png.len() > 200, "{name} drew something");
        }
    }

    #[test]
    fn shapes_match_the_apps_geometry() {
        // shapes.ts: the first point of cookie9 is at θ = 0, r = 20 · 1.09.
        assert!(shape_path("cookie9").starts_with("M45.80 24.00 "));
        assert!(shape_path("circle").starts_with("M44.00 24.00 "));
    }

    #[test]
    fn lighten_mixes_toward_white() {
        assert_eq!(lighten("#000000", 0.5), "#808080");
        assert_eq!(lighten("#ff7a4a", 0.0), "#ff7a4a");
        assert_eq!(lighten("#ff7a4a", 1.0), "#ffffff");
    }
}
