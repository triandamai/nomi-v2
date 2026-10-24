package ai.nomi.feature.chat.views

import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.booleanOrNull
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.doubleOrNull
import kotlinx.serialization.json.intOrNull

// Views: what the crew shows in a chat bubble when plain text isn't the best fit (`{kind: "view", view: …}` blocks).
// Mirrors frontend/src/lib/views/types.ts and the backend's catalog (nomi-agent-core/src/views.rs); the web's
// examples.json is parsed by ViewParityTest so the sides can't drift.

/** A value as the server sent it: text, a number or a yes/no. */
sealed interface Scalar {
    data class Text(val value: String) : Scalar
    data class Num(val value: Double) : Scalar
    data class Bool(val value: Boolean) : Scalar
}

/** How a value is shown (`Format` on the web). */
enum class ValueFormat(val wire: String) {
    Text("text"), Number("number"), Money("money"), Percent("percent"), Date("date"), DateTime("date_time");

    companion object {
        fun of(wire: String?): ValueFormat? = entries.firstOrNull { it.wire == wire }
    }
}

enum class Tone(val wire: String) {
    Neutral("neutral"), Positive("positive"), Negative("negative"), Warning("warning");

    companion object {
        fun of(wire: String?): Tone = entries.firstOrNull { it.wire == wire } ?: Neutral
    }
}

data class Field(val label: String, val value: Scalar, val format: ValueFormat? = null)

/** One view; [name] is its `view` value. */
sealed interface ChatView {
    val name: String

    data class ListItem(val title: String, val detail: String?, val meta: Scalar?, val format: ValueFormat?, val done: Boolean, val url: String?)
    enum class ListStyle { Bullet, Numbered, Check }
    data class List(val title: String?, val style: ListStyle, val items: kotlin.collections.List<ListItem>) : ChatView { override val name get() = "list" }

    data class Column(val key: String, val label: String, val format: ValueFormat?)
    data class Table(val title: String?, val columns: kotlin.collections.List<Column>, val rows: kotlin.collections.List<Map<String, Scalar?>>) : ChatView {
        override val name get() = "table"
    }

    data class ComparisonRow(val label: String, val values: kotlin.collections.List<Scalar?>)
    data class Comparison(
        val title: String?,
        val options: kotlin.collections.List<String>,
        val rows: kotlin.collections.List<ComparisonRow>,
        val recommended: Int?,
        val verdict: String?,
    ) : ChatView { override val name get() = "comparison" }

    enum class StepStatus(val wire: String, val spoken: String) {
        Done("done", "done"), Current("current", "in progress"), Upcoming("upcoming", "to do"), Failed("failed", "failed"), Skipped("skipped", "skipped");

        companion object {
            fun of(wire: String?) = entries.firstOrNull { it.wire == wire } ?: Upcoming
        }
    }
    data class Step(val title: String, val detail: String?, val status: StepStatus)
    data class Stepper(val title: String?, val steps: kotlin.collections.List<Step>) : ChatView { override val name get() = "stepper" }

    data class Event(val at: String, val title: String, val detail: String?)
    data class Timeline(val title: String?, val events: kotlin.collections.List<Event>) : ChatView { override val name get() = "timeline" }

    data class Link(val label: String, val url: String)
    data class Action(val label: String, val message: String)
    data class Card(
        val eyebrow: String?,
        val title: String,
        val subtitle: String?,
        val highlight: Field?,
        val fields: kotlin.collections.List<Field>,
        val body: String?,
        val link: Link?,
        val actions: kotlin.collections.List<Action>,
        val tone: Tone,
    ) : ChatView { override val name get() = "card" }

    enum class ChartKind { Bar, Line, Donut }
    data class Series(val name: String, val values: kotlin.collections.List<Double>)
    data class Chart(
        val title: String?,
        val kind: ChartKind,
        val labels: kotlin.collections.List<String>,
        val series: kotlin.collections.List<Series>,
        val format: ValueFormat?,
    ) : ChatView { override val name get() = "chart" }

    data class Answer(val label: String?, val value: Scalar, val format: ValueFormat?, val detail: String?, val tone: Tone) : ChatView {
        override val name get() = "answer"
    }

    data class Suggestions(val title: String?, val items: kotlin.collections.List<String>) : ChatView { override val name get() = "suggestions" }

    data class Option(val label: String, val detail: String?, val pros: kotlin.collections.List<String>, val cons: kotlin.collections.List<String>, val recommended: Boolean)
    data class Decision(val question: String, val options: kotlin.collections.List<Option>) : ChatView { override val name get() = "decision" }

    enum class FieldType(val wire: String) {
        Text("text"), Textarea("textarea"), Number("number"), Date("date"), Time("time"), Select("select"), Multiselect("multiselect"), Checkbox("checkbox");

        companion object {
            fun of(wire: String?) = entries.firstOrNull { it.wire == wire } ?: Text
        }
    }
    data class FormField(
        val key: String,
        val label: String,
        val type: FieldType,
        val required: Boolean,
        val options: kotlin.collections.List<String>,
        val placeholder: String?,
        val default: Scalar?,
    )
    data class Form(val title: String?, val intro: String?, val fields: kotlin.collections.List<FormField>, val submitLabel: String?) : ChatView {
        override val name get() = "form"
    }

    data class LinkPreview(val url: String, val title: String?, val description: String?, val image: String?, val site: String?) : ChatView {
        override val name get() = "link"
    }

    enum class TaskStatus(val wire: String, val label: String) {
        Queued("queued", "Queued"), Running("running", "Running"), Done("done", "Done"), Failed("failed", "Failed");

        companion object {
            fun of(wire: String?) = entries.firstOrNull { it.wire == wire } ?: Queued
        }
    }
    data class Task(val title: String, val status: TaskStatus, val detail: String?, val progress: Double?, val agent: String?) : ChatView {
        override val name get() = "task"
    }

    data class Shell(val command: String, val output: String?, val exitCode: Int?, val cwd: String?) : ChatView { override val name get() = "shell" }
}

/** Every view this build draws (`VIEW_NAMES` on the web). */
val ViewNames = listOf("list", "table", "comparison", "stepper", "timeline", "card", "chart", "answer", "suggestions", "decision", "form", "link", "task", "shell")

/**
 * A view from its block (the `view` field names it). Null for a view this build doesn't know or one missing what it
 * needs — the message's own text then stands in, as on the web.
 */
fun parseView(block: JsonObject): ChatView? = runCatching { block.view() }.getOrNull()

private fun JsonObject.view(): ChatView? = when (string("view")) {
    "list" -> ChatView.List(
        string("title"),
        when (string("style")) { "numbered" -> ChatView.ListStyle.Numbered; "check" -> ChatView.ListStyle.Check; else -> ChatView.ListStyle.Bullet },
        objects("items").map {
            ChatView.ListItem(it.require("title"), it.string("detail"), it.scalar("meta"), ValueFormat.of(it.string("format")), it.bool("done") == true, it.string("url"))
        },
    )
    "table" -> {
        val columns = objects("columns").map { ChatView.Column(it.require("key"), it.require("label"), ValueFormat.of(it.string("format"))) }
        ChatView.Table(string("title"), columns, objects("rows").map { row -> columns.associate { it.key to row.scalar(it.key) } })
    }
    "comparison" -> ChatView.Comparison(
        string("title"),
        strings("options"),
        objects("rows").map { row -> ChatView.ComparisonRow(row.require("label"), (row["values"] as? JsonArray).orEmpty().map { it.toScalar() }) },
        int("recommended"),
        string("verdict"),
    )
    "stepper" -> ChatView.Stepper(string("title"), objects("steps").map { ChatView.Step(it.require("title"), it.string("detail"), ChatView.StepStatus.of(it.string("status"))) })
    "timeline" -> ChatView.Timeline(string("title"), objects("events").map { ChatView.Event(it.require("at"), it.require("title"), it.string("detail")) })
    "card" -> ChatView.Card(
        string("eyebrow"),
        require("title"),
        string("subtitle"),
        obj("highlight")?.field(),
        objects("fields").mapNotNull { it.field() },
        string("body"),
        obj("link")?.let { ChatView.Link(it.require("label"), it.require("url")) },
        objects("actions").map { ChatView.Action(it.require("label"), it.require("message")) },
        Tone.of(string("tone")),
    )
    "chart" -> ChatView.Chart(
        string("title"),
        when (string("chart")) { "line" -> ChatView.ChartKind.Line; "donut" -> ChatView.ChartKind.Donut; else -> ChatView.ChartKind.Bar },
        strings("labels"),
        objects("series").map { s -> ChatView.Series(s.require("name"), (s["values"] as? JsonArray).orEmpty().map { (it as? JsonPrimitive)?.doubleOrNull ?: 0.0 }) },
        ValueFormat.of(string("format")),
    )
    "answer" -> ChatView.Answer(string("label"), scalar("value") ?: return null, ValueFormat.of(string("format")), string("detail"), Tone.of(string("tone")))
    "suggestions" -> ChatView.Suggestions(string("title"), strings("items"))
    "decision" -> ChatView.Decision(
        require("question"),
        objects("options").map { ChatView.Option(it.require("label"), it.string("detail"), it.strings("pros"), it.strings("cons"), it.bool("recommended") == true) },
    )
    "form" -> ChatView.Form(
        string("title"),
        string("intro"),
        objects("fields").map {
            ChatView.FormField(it.require("key"), it.require("label"), ChatView.FieldType.of(it.string("type")), it.bool("required") == true, it.strings("options"), it.string("placeholder"), it.scalar("default"))
        },
        string("submit_label"),
    )
    "link" -> ChatView.LinkPreview(require("url"), string("title"), string("description"), string("image"), string("site"))
    "task" -> ChatView.Task(require("title"), ChatView.TaskStatus.of(string("status")), string("detail"), double("progress"), string("agent"))
    "shell" -> ChatView.Shell(require("command"), string("output"), int("exit_code"), string("cwd"))
    else -> null
}

/** A `{label, value, format}` field; null when it has no value. */
private fun JsonObject.field(): Field? = Field(require("label"), scalar("value") ?: return null, ValueFormat.of(string("format")))

internal fun JsonElement?.toScalar(): Scalar? {
    val primitive = this as? JsonPrimitive ?: return null
    if (primitive is JsonNull) return null
    if (primitive.isString) return Scalar.Text(primitive.content)
    primitive.booleanOrNull?.let { return Scalar.Bool(it) }
    return primitive.doubleOrNull?.let { Scalar.Num(it) }
}

internal fun JsonObject.string(key: String): String? = (this[key] as? JsonPrimitive)?.takeIf { it.isString }?.contentOrNull
private fun JsonObject.require(key: String): String = string(key) ?: error("missing '$key'")
private fun JsonObject.bool(key: String): Boolean? = (this[key] as? JsonPrimitive)?.booleanOrNull
private fun JsonObject.int(key: String): Int? = (this[key] as? JsonPrimitive)?.intOrNull
private fun JsonObject.double(key: String): Double? = (this[key] as? JsonPrimitive)?.doubleOrNull
private fun JsonObject.scalar(key: String): Scalar? = this[key].toScalar()
private fun JsonObject.obj(key: String): JsonObject? = this[key] as? JsonObject
internal fun JsonObject.objects(key: String): List<JsonObject> = (this[key] as? JsonArray).orEmpty().mapNotNull { it as? JsonObject }
internal fun JsonObject.strings(key: String): List<String> = (this[key] as? JsonArray).orEmpty().mapNotNull { (it as? JsonPrimitive)?.takeIf { p -> p.isString }?.content }
