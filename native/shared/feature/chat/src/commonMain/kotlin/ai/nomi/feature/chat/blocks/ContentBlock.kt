package ai.nomi.feature.chat.blocks

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.booleanOrNull
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.intOrNull

/**
 * One structured piece of a message (`ContentBlock` in `frontend/src/lib/types.ts`, the server's
 * `content_blocks`). Only the kinds the native chat draws are modelled; every other kind is [Unknown] and shows the
 * message's own text in a quiet card.
 */
sealed interface ContentBlock {
    /** A gated tool call waiting for (or past) the person's OK. */
    data class Approval(
        val id: String,
        val toolName: String,
        val description: String,
        val status: ApprovalStatus,
    ) : ContentBlock

    /** The agent's to-do list, updated in place as it works. */
    data class TodoList(val items: List<TodoItem>) : ContentBlock

    /** A version of an agent's plan; its checklist lives with the plan, not the message. */
    data class Plan(val planId: String, val agentSessionId: String, val title: String, val version: Int) : ContentBlock

    /** A file the agent created, changed or deleted ([content] is null for a delete). */
    data class FileChange(
        val path: String,
        val action: FileAction,
        val content: String?,
        val previousContent: String?,
    ) : ContentBlock

    /** The model's thinking. */
    data class Reasoning(val text: String) : ContentBlock

    /** A view — a list, table, chart, form… (`{kind: "view"}`, see the views package). */
    data class View(val view: ai.nomi.feature.chat.views.ChatView) : ContentBlock

    /** Questions to answer by tapping options (`questions`). */
    data class Questions(val block: ai.nomi.feature.chat.views.Questions) : ContentBlock

    /** A kind this build doesn't draw (or one that arrived malformed). */
    data class Unknown(val kind: String) : ContentBlock
}

enum class ApprovalStatus(val wire: String) {
    Pending("pending"),
    Approved("approved"),
    Denied("denied"),

    /** The person stopped the agent before deciding. */
    Cancelled("cancelled"),
}

enum class TodoStatus(val wire: String) { Pending("pending"), InProgress("in_progress"), Done("done") }

data class TodoItem(val id: String, val text: String, val status: TodoStatus)

enum class FileAction { Created, Updated, Deleted }

private const val KIND = "kind"
private const val KIND_APPROVAL = "approval_request"
private const val KIND_TODO = "todo_list"
private const val KIND_PLAN = "plan"
private const val KIND_FILE_WRITE = "file_write"
private const val KIND_FILE_WRITES = "file_writes"
private const val KIND_FILE_DELETE = "file_delete"
private const val KIND_REASONING = "reasoning"
private const val KIND_VIEW = "view"
private const val KIND_TABLE = "table"
private const val KIND_QUESTIONS = "questions"
private const val UNNAMED_KIND = "unknown"

private val lenient = Json { ignoreUnknownKeys = true }

/**
 * The blocks in a message's [json] (its `contentJson`), in order. Null, blank or unreadable JSON gives no blocks (the
 * message's text is shown instead); a block missing a field it needs becomes [ContentBlock.Unknown]. A `file_writes`
 * batch becomes one [ContentBlock.FileChange] per file.
 */
fun parseContentBlocks(json: String?): List<ContentBlock> {
    if (json.isNullOrBlank()) return emptyList()
    val array = runCatching { lenient.parseToJsonElement(json) }.getOrNull() as? JsonArray ?: return emptyList()
    return array.flatMap { element -> (element as? JsonObject)?.let(::parseBlock) ?: emptyList() }
}

private fun parseBlock(block: JsonObject): List<ContentBlock> {
    val kind = block.string(KIND) ?: UNNAMED_KIND
    val parsed: List<ContentBlock>? = when (kind) {
        KIND_APPROVAL -> block.approval()?.let(::listOf)
        KIND_TODO -> block.todoList()?.let(::listOf)
        KIND_PLAN -> block.plan()?.let(::listOf)
        KIND_FILE_WRITE -> block.fileWrite()?.let(::listOf)
        KIND_FILE_WRITES -> block.fileWrites()
        KIND_FILE_DELETE -> block.string("path")?.let { listOf(ContentBlock.FileChange(it, FileAction.Deleted, null, block.string("previous_content"))) }
        KIND_REASONING -> block.string("text")?.let { listOf(ContentBlock.Reasoning(it)) }
        KIND_VIEW -> ai.nomi.feature.chat.views.parseView(block)?.let { listOf(ContentBlock.View(it)) }
        // The older table block draws as a table view (its values are already formatted text).
        KIND_TABLE -> ai.nomi.feature.chat.views.parseView(JsonObject(block + ("view" to JsonPrimitive(KIND_TABLE))))?.let { listOf(ContentBlock.View(it)) }
        KIND_QUESTIONS -> block.questions()?.let { listOf(ContentBlock.Questions(it)) }
        else -> null
    }
    return parsed ?: listOf(ContentBlock.Unknown(kind))
}

private fun JsonObject.approval(): ContentBlock.Approval? {
    val status = string("status")?.let { wire -> ApprovalStatus.entries.firstOrNull { it.wire == wire } } ?: return null
    return ContentBlock.Approval(
        id = string("id") ?: return null,
        toolName = string("tool_name") ?: return null,
        description = string("description") ?: return null,
        status = status,
    )
}

private fun JsonObject.todoList(): ContentBlock.TodoList? {
    val items = this["items"] as? JsonArray ?: return null
    return ContentBlock.TodoList(
        items.mapIndexedNotNull { index, element ->
            val item = element as? JsonObject ?: return@mapIndexedNotNull null
            val text = item.string("text") ?: return@mapIndexedNotNull null
            // A status this build doesn't know reads as not started rather than hiding the step.
            val status = TodoStatus.entries.firstOrNull { it.wire == item.string("status") } ?: TodoStatus.Pending
            TodoItem(item.string("id") ?: index.toString(), text, status)
        },
    )
}

private fun JsonObject.plan(): ContentBlock.Plan? = ContentBlock.Plan(
    planId = string("plan_id") ?: return null,
    agentSessionId = string("agent_session_id") ?: return null,
    title = string("title") ?: return null,
    version = (this["version"] as? JsonPrimitive)?.intOrNull ?: return null,
)

private fun JsonObject.fileWrite(): ContentBlock.FileChange? {
    val path = string("path") ?: return null
    val content = string("content") ?: return null
    val previous = string("previous_content")
    return ContentBlock.FileChange(path, if (previous == null) FileAction.Created else FileAction.Updated, content, previous)
}

private fun JsonObject.fileWrites(): List<ContentBlock>? {
    val files = this["files"] as? JsonArray ?: return null
    return files.mapNotNull { (it as? JsonObject)?.fileWrite() }
}

private fun JsonObject.questions(): ai.nomi.feature.chat.views.Questions? {
    val questions = (this["questions"] as? JsonArray)?.mapNotNull { element ->
        val q = element as? JsonObject ?: return@mapNotNull null
        val options = (q["options"] as? JsonArray).orEmpty().mapNotNull { (it as? JsonObject)?.let { o -> o.string("label")?.let { l -> l to o.string("detail") } } }
        ai.nomi.feature.chat.views.Question(q.string("question") ?: return@mapNotNull null, options, (q["multiple"] as? JsonPrimitive)?.booleanOrNull == true)
    } ?: return null
    return ai.nomi.feature.chat.views.Questions(string("intro"), questions)
}

/** A string field; JSON null, a missing key or a non-string value is null. */
private fun JsonObject.string(key: String): String? = (this[key] as? JsonPrimitive)?.takeIf { it.isString }?.contentOrNull
