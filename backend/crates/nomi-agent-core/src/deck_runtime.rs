//! The deck runtime Sora's slide decks run on: html-ppt-skill's runtime, base styles, animations
//! and themes (MIT, Copyright (c) 2026 lewis; its LICENSE travels with it), plus Nomi's bridge from
//! its theme variables to a design's own tokens. A deck links these under `{PREFIX}` (relative to
//! the deck), so the same files are served with every preview and copied into every published deck.

use crate::design_artifact::ArtifactFile;

/// Where a deck finds the runtime, relative to its own `index.html`.
pub const PREFIX: &str = "_runtime/html-ppt/";

/// The runtime's files: (path under `PREFIX`, contents).
pub static FILES: &[(&str, &str)] = &[
    ("LICENSE", include_str!("../assets/deck-runtime/html-ppt/LICENSE")),
    ("runtime.js", include_str!("../assets/deck-runtime/html-ppt/runtime.js")),
    ("base.css", include_str!("../assets/deck-runtime/html-ppt/base.css")),
    ("animations.css", include_str!("../assets/deck-runtime/html-ppt/animations.css")),
    ("nomi-bridge.css", include_str!("../assets/deck-runtime/html-ppt/nomi-bridge.css")),
    ("themes/academic-paper.css", include_str!("../assets/deck-runtime/html-ppt/themes/academic-paper.css")),
    ("themes/arctic-cool.css", include_str!("../assets/deck-runtime/html-ppt/themes/arctic-cool.css")),
    ("themes/aurora.css", include_str!("../assets/deck-runtime/html-ppt/themes/aurora.css")),
    ("themes/bauhaus.css", include_str!("../assets/deck-runtime/html-ppt/themes/bauhaus.css")),
    ("themes/blueprint.css", include_str!("../assets/deck-runtime/html-ppt/themes/blueprint.css")),
    ("themes/catppuccin-latte.css", include_str!("../assets/deck-runtime/html-ppt/themes/catppuccin-latte.css")),
    ("themes/catppuccin-mocha.css", include_str!("../assets/deck-runtime/html-ppt/themes/catppuccin-mocha.css")),
    ("themes/corporate-clean.css", include_str!("../assets/deck-runtime/html-ppt/themes/corporate-clean.css")),
    ("themes/cyberpunk-neon.css", include_str!("../assets/deck-runtime/html-ppt/themes/cyberpunk-neon.css")),
    ("themes/dracula.css", include_str!("../assets/deck-runtime/html-ppt/themes/dracula.css")),
    ("themes/editorial-serif.css", include_str!("../assets/deck-runtime/html-ppt/themes/editorial-serif.css")),
    ("themes/engineering-whiteprint.css", include_str!("../assets/deck-runtime/html-ppt/themes/engineering-whiteprint.css")),
    ("themes/glassmorphism.css", include_str!("../assets/deck-runtime/html-ppt/themes/glassmorphism.css")),
    ("themes/gruvbox-dark.css", include_str!("../assets/deck-runtime/html-ppt/themes/gruvbox-dark.css")),
    ("themes/japanese-minimal.css", include_str!("../assets/deck-runtime/html-ppt/themes/japanese-minimal.css")),
    ("themes/magazine-bold.css", include_str!("../assets/deck-runtime/html-ppt/themes/magazine-bold.css")),
    ("themes/memphis-pop.css", include_str!("../assets/deck-runtime/html-ppt/themes/memphis-pop.css")),
    ("themes/midcentury.css", include_str!("../assets/deck-runtime/html-ppt/themes/midcentury.css")),
    ("themes/minimal-white.css", include_str!("../assets/deck-runtime/html-ppt/themes/minimal-white.css")),
    ("themes/neo-brutalism.css", include_str!("../assets/deck-runtime/html-ppt/themes/neo-brutalism.css")),
    ("themes/news-broadcast.css", include_str!("../assets/deck-runtime/html-ppt/themes/news-broadcast.css")),
    ("themes/nord.css", include_str!("../assets/deck-runtime/html-ppt/themes/nord.css")),
    ("themes/pitch-deck-vc.css", include_str!("../assets/deck-runtime/html-ppt/themes/pitch-deck-vc.css")),
    ("themes/rainbow-gradient.css", include_str!("../assets/deck-runtime/html-ppt/themes/rainbow-gradient.css")),
    ("themes/retro-tv.css", include_str!("../assets/deck-runtime/html-ppt/themes/retro-tv.css")),
    ("themes/rose-pine.css", include_str!("../assets/deck-runtime/html-ppt/themes/rose-pine.css")),
    ("themes/sharp-mono.css", include_str!("../assets/deck-runtime/html-ppt/themes/sharp-mono.css")),
    ("themes/soft-pastel.css", include_str!("../assets/deck-runtime/html-ppt/themes/soft-pastel.css")),
    ("themes/solarized-light.css", include_str!("../assets/deck-runtime/html-ppt/themes/solarized-light.css")),
    ("themes/sunset-warm.css", include_str!("../assets/deck-runtime/html-ppt/themes/sunset-warm.css")),
    ("themes/swiss-grid.css", include_str!("../assets/deck-runtime/html-ppt/themes/swiss-grid.css")),
    ("themes/terminal-green.css", include_str!("../assets/deck-runtime/html-ppt/themes/terminal-green.css")),
    ("themes/tokyo-night.css", include_str!("../assets/deck-runtime/html-ppt/themes/tokyo-night.css")),
    ("themes/vaporwave.css", include_str!("../assets/deck-runtime/html-ppt/themes/vaporwave.css")),
    ("themes/xiaohongshu-white.css", include_str!("../assets/deck-runtime/html-ppt/themes/xiaohongshu-white.css")),
    ("themes/y2k-chrome.css", include_str!("../assets/deck-runtime/html-ppt/themes/y2k-chrome.css")),
];

/// The runtime file a deck asked for at `path` (a path under the deck); `None` when it isn't one.
pub fn file(path: &str) -> Option<&'static str> {
    let name = path.strip_prefix(PREFIX)?;
    FILES.iter().find(|(file, _)| *file == name).map(|(_, text)| *text)
}

/// The runtime as files of a deck, to publish with it.
pub fn artifact_files() -> Vec<ArtifactFile> {
    FILES.iter().map(|(name, text)| ArtifactFile { path: format!("{PREFIX}{name}"), content: text.to_string() }).collect()
}

/// A version's files as they are published: a deck takes the runtime (and its licence) along, so it
/// works wherever it is opened; anything else is published as it is.
pub fn publishable_files(kind: crate::design_artifact::Kind, files: &[ArtifactFile]) -> Vec<ArtifactFile> {
    let mut out = files.to_vec();
    if kind == crate::design_artifact::Kind::Deck {
        out.extend(artifact_files());
    }
    out
}

/// The theme names a deck can link (`themes/<name>.css`).
pub fn themes() -> Vec<&'static str> {
    FILES.iter().filter_map(|(name, _)| name.strip_prefix("themes/")?.strip_suffix(".css")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_deck_finds_the_runtime_and_its_licence_under_the_prefix() {
        assert!(file("_runtime/html-ppt/runtime.js").unwrap().contains("BroadcastChannel"));
        assert!(file("_runtime/html-ppt/base.css").is_some() && file("_runtime/html-ppt/animations.css").is_some());
        assert!(file("_runtime/html-ppt/LICENSE").unwrap().contains("MIT License"), "the MIT notice travels with the runtime");
        assert_eq!(file("index.html"), None);
        assert_eq!(file("_runtime/html-ppt/../secret"), None);
    }

    #[test]
    fn there_are_themes_to_link_and_a_bridge_to_the_designs_own_tokens() {
        assert!(themes().len() >= 30, "{} themes", themes().len());
        assert!(file("_runtime/html-ppt/themes/academic-paper.css").is_some());
        let bridge = file("_runtime/html-ppt/nomi-bridge.css").unwrap();
        assert!(bridge.contains("--accent: var(--color-primary)") && bridge.contains("--bg: var(--color-background)"));
    }

    #[test]
    fn only_a_deck_is_published_with_the_runtime() {
        use crate::design_artifact::Kind;
        let own = vec![ArtifactFile { path: "index.html".into(), content: "<div class=\"deck\"></div>".into() }];
        let deck = publishable_files(Kind::Deck, &own);
        assert_eq!(deck.len(), own.len() + FILES.len());
        assert_eq!(deck[0], own[0], "the deck's own files come first");
        assert_eq!(publishable_files(Kind::Design, &own), own, "a design is published as it is");
        assert_eq!(publishable_files(Kind::Video, &own), own);
    }

    #[test]
    fn a_published_deck_carries_every_runtime_file() {
        let files = artifact_files();
        assert_eq!(files.len(), FILES.len());
        assert!(files.iter().all(|f| f.path.starts_with(PREFIX)));
        assert!(files.iter().any(|f| f.path == "_runtime/html-ppt/LICENSE"));
    }
}
