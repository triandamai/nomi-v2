//! Koda's built-in coding knowledge: one guide per stack (checked against real installs and
//! builds), plus the database (Drizzle), Relay and preview guides. They're compiled in and read on demand with
//! read_guide, so the system prompt stays short.

use serde_json::{json, Value};

use nomi_llm::ToolDefinition;

/// (topic, what it covers, content).
pub const GUIDES: &[(&str, &str, &str)] = &[
    ("sveltekit", "SvelteKit 3 + Svelte 5: full-stack apps (the default)", include_str!("../guides/sveltekit.md")),
    ("design", "How every project should look: a deliberate modern style, and the generic AI look to avoid (an approved design from Sora wins over it)", include_str!("../guides/design.md")),
    ("components", "Building the UI with shadcn-svelte (shadcn/ui in React): setup, theming from the design tokens, adding components without the CLI, and Sora's component names", include_str!("../guides/components.md")),
    ("database", "Drizzle with Postgres or SQLite, and how the preview runs them (projects not on Relay)", include_str!("../guides/database.md")),
    ("relay", "Nomi Relay: migrations with row-level security, the nomi-relay client, sign-in, files, live changes and hosted functions (projects on Relay)", include_str!("../guides/relay.md")),
    ("preview", "How Nomi runs the project in the browser, and what can't run there", include_str!("../guides/preview.md")),
    ("sandbox", "How it differs when the project runs in a sandbox on Nomi's servers", include_str!("../guides/sandbox.md")),
    ("react", "React 19 + Vite single-page apps", include_str!("../guides/react.md")),
    ("vue", "Vue 3 + Vite single-page apps", include_str!("../guides/vue.md")),
    ("astro", "Astro 7 content sites with Svelte islands", include_str!("../guides/astro.md")),
];

/// The guide for a project's stack, if it has one.
pub fn guide_for_stack(stack: &str) -> Option<&'static str> {
    match stack {
        "sveltekit" | "svelte" => Some("sveltekit"),
        "react" => Some("react"),
        "vue" => Some("vue"),
        "astro" => Some("astro"),
        _ => None,
    }
}

pub fn read_guide_tool_definition() -> ToolDefinition {
    let topics: Vec<&str> = GUIDES.iter().map(|(topic, _, _)| *topic).collect();
    let listing = GUIDES.iter().map(|(topic, about, _)| format!("{topic}: {about}")).collect::<Vec<_>>().join("; ");
    ToolDefinition {
        name: "read_guide".to_string(),
        description: format!(
            "Read one of your coding guides: exact package versions, config files and conventions that are known to work in Nomi. Read the stack's guide before writing a project's first files, the design guide and (in SvelteKit or React) the components guide before writing any UI, and the database guide before adding a database. Guides: {listing}."
        ),
        input_schema: json!({
            "type": "object",
            "properties": {"topic": {"type": "string", "enum": topics}},
            "required": ["topic"]
        }),
    }
}

pub fn read_guide(input: &Value) -> Result<String, String> {
    let topic = input.get("topic").and_then(|v| v.as_str()).ok_or("topic is required")?.trim().to_lowercase();
    GUIDES
        .iter()
        .find(|(name, _, _)| *name == topic)
        .map(|(_, _, content)| content.to_string())
        .ok_or_else(|| format!("no guide called {topic}; there are: {}", GUIDES.iter().map(|g| g.0).collect::<Vec<_>>().join(", ")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_stack_with_a_guide_points_at_a_real_one() {
        for stack in ["sveltekit", "svelte", "react", "vue", "astro"] {
            let topic = guide_for_stack(stack).unwrap();
            assert!(read_guide(&json!({ "topic": topic })).unwrap().len() > 500, "{stack}");
        }
        assert!(guide_for_stack("static").is_none());
        assert!(read_guide(&json!({ "topic": "cobol" })).is_err());
    }

    #[test]
    fn the_design_guide_names_the_generic_look_to_avoid() {
        let guide = read_guide(&json!({ "topic": "design" })).unwrap();
        assert!(guide.contains("AI slop") && guide.contains("@theme") && guide.contains("complete_task"));
    }

    #[test]
    fn the_sandbox_guide_says_where_it_runs_and_keeps_the_browser_rules() {
        let guide = read_guide(&json!({ "topic": "sandbox" })).unwrap();
        assert!(guide.contains("sandbox on Nomi's servers"), "{guide}");
        assert!(guide.contains("doesn't need the project open"), "{guide}");
        assert!(guide.contains("still starting") && guide.contains("couldn't start"), "{guide}");
        assert!(guide.contains("back to the browser"), "the preview guide's rules still hold: {guide}");
        let preview = read_guide(&json!({ "topic": "preview" })).unwrap();
        assert!(preview.contains("sandbox guide"), "the preview guide points at it: {preview}");
    }

    #[test]
    fn the_components_guide_builds_with_shadcn_svelte_and_says_how_to_add_its_parts() {
        let guide = read_guide(&json!({ "topic": "components" })).unwrap();
        assert!(guide.contains("shadcn-svelte") && guide.contains("shadcn/ui"), "{guide}");
        assert!(guide.contains("npx shadcn-svelte@latest init") && guide.contains("npx shadcn-svelte@latest add"), "names the real CLI: {guide}");
        assert!(guide.contains("don't come back into the project"), "says why the CLI isn't used in Nomi: {guide}");
        assert!(guide.contains("https://shadcn-svelte.com/registry/") && guide.contains("write_files"), "and how to add components instead: {guide}");
        assert!(guide.contains("#lib/components/ui/button/index.ts") && guide.contains("#lib/utils.ts"), "with SvelteKit 3's #lib imports: {guide}");
        assert!(guide.contains("@theme inline") && guide.contains("--primary-foreground"), "themed from the design tokens: {guide}");
    }

    #[test]
    fn the_components_guide_maps_soras_component_names_and_keeps_off_rare_ui_and_beui() {
        let guide = read_guide(&json!({ "topic": "components" })).unwrap();
        assert!(guide.contains("<Area> / <Thing> / <State>") && guide.contains("data-design-id"), "{guide}");
        for name in ["Button", "Card", "Dialog", "Sheet", "Sonner", "Tabs", "Select", "Table", "Badge", "Avatar"] {
            assert!(guide.contains(name), "maps {name}: {guide}");
        }
        assert!(guide.contains("variant=\"outline\"") && guide.contains("destructive") && guide.contains("ghost"), "{guide}");
        assert!(guide.contains("rare-ui") && guide.contains("beUI") && guide.contains("Commons Clause"), "{guide}");
    }

    #[test]
    fn koda_is_told_to_read_the_components_guide_and_the_preview_guide_allows_it() {
        let tool = read_guide_tool_definition();
        assert!(tool.description.contains("the components guide before writing any UI"), "{}", tool.description);
        let preview = read_guide(&json!({ "topic": "preview" })).unwrap();
        assert!(!preview.contains("No other CSS frameworks or component kits"), "{preview}");
        assert!(preview.contains("components guide"), "{preview}");
    }

    #[test]
    fn the_design_guide_lets_an_approved_design_win_over_every_rule_including_its_list() {
        let guide = read_guide(&json!({ "topic": "design" })).unwrap();
        let wins = guide.find("that design wins over every rule in this guide, including the \"never do these\" list").expect("says the approved design wins");
        assert!(guide.contains("The list applies only when there is no approved design"), "{guide}");
        assert!(wins < guide.find("(never do these)").unwrap(), "said before the list: {guide}");
        let checklist = &guide[guide.find("## Before you call complete_task").unwrap()..];
        assert!(checklist.contains("approved design"), "the checklist doesn't undo it: {checklist}");
    }

    #[test]
    fn the_sveltekit_guide_teaches_version_3() {
        let guide = read_guide(&json!({ "topic": "sveltekit" })).unwrap();
        assert!(guide.contains("$app/env/private") && guide.contains("#lib/server/db/index.ts"));
    }
}
