package ai.nomi.designsystem.navigation

import ai.nomi.designsystem.icon.NomiGlyph
import ai.nomi.designsystem.icon.NomiIcons

data class NomiFeature(
    val id: String,
    val label: String,
    val description: String,
    val glyph: NomiGlyph,
    val isPermanent: Boolean = false,
)

object NomiFeatures {
    val PermanentIds = listOf("home", "chats", "projects")
    val PinnableIds = listOf("money", "reminders", "memory", "notifications", "crew", "connections", "models", "billing", "showcase")
    val DefaultPins = listOf("money", "reminders", "memory", "notifications")

    val All: Map<String, NomiFeature> = mapOf(
        "home" to NomiFeature("home", "Home", "Start new chats, review your day and see updates", NomiIcons.Home, isPermanent = true),
        "chats" to NomiFeature("chats", "Chats", "Your conversations with Nomi and the crew", NomiIcons.Chat, isPermanent = true),
        "projects" to NomiFeature("projects", "Projects", "Long-running workspaces, documents and context", NomiIcons.Folder, isPermanent = true),
        "showcase" to NomiFeature("showcase", "Showcase", "Community agents, workflows and inspirations", NomiIcons.Sparkle),
        "money" to NomiFeature("money", "Money", "Track accounts, recurring charges and daily burn rate", NomiIcons.Wallet),
        "reminders" to NomiFeature("reminders", "Reminders", "Tasks and reminders across chat and calendar", NomiIcons.Clock),
        "memory" to NomiFeature("memory", "Memory", "Facts, preferences and context Nomi remembers", NomiIcons.Memory),
        "notifications" to NomiFeature("notifications", "Inbox", "Approvals, scheduled tasks and unread pings", NomiIcons.Bell),
        "crew" to NomiFeature("crew", "Crew", "Specialist agents working alongside Nomi", NomiIcons.Person),
        "connections" to NomiFeature("connections", "Connections", "Integrations with tools and external services", NomiIcons.Link),
        "models" to NomiFeature("models", "Models", "Model picker, fallbacks and custom keys", NomiIcons.Sparkle),
        "billing" to NomiFeature("billing", "Billing & Usage", "Plan limits, tokens used and invoices", NomiIcons.Card),
        "preferences" to NomiFeature("preferences", "Preferences", "Themes, display options and default crew", NomiIcons.Settings),
        "profile" to NomiFeature("profile", "Profile", "Personal details and account security", NomiIcons.Person),
    )

    data class Section(val title: String, val ids: List<String>)

    val Sections = listOf(
        Section("Main", listOf("home", "chats", "projects", "showcase")),
        Section("Life", listOf("money", "reminders", "memory", "notifications")),
        Section("Crew", listOf("crew", "connections", "models")),
        Section("Account", listOf("billing", "preferences", "profile")),
    )
}
