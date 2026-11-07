package ai.nomi.feature.chat.blocks

import ai.nomi.designsystem.ds.Crew
import ai.nomi.designsystem.ds.Ds
import ai.nomi.designsystem.ds.StepState
import ai.nomi.designsystem.ds.builds.TermLine
import ai.nomi.designsystem.ds.builds.TermLineKind
import ai.nomi.designsystem.ds.builds.Terminal
import ai.nomi.designsystem.ds.chat.ApprovalCard as DsApprovalCard
import ai.nomi.designsystem.ds.chat.ApprovalStatus as DsApprovalStatus
import ai.nomi.designsystem.ds.chat.Diff
import ai.nomi.designsystem.ds.chat.DiffHunk
import ai.nomi.designsystem.ds.chat.DiffLine
import ai.nomi.designsystem.ds.chat.DiffLineKind
import ai.nomi.designsystem.ds.chat.FollowUps
import ai.nomi.designsystem.ds.chat.KeyValue
import ai.nomi.designsystem.ds.chat.LinkPreview
import ai.nomi.designsystem.ds.chat.PlanCard as DsPlanCard
import ai.nomi.designsystem.ds.chat.PlanStep
import ai.nomi.designsystem.ds.chat.Reasoning
import ai.nomi.designsystem.ds.chat.StepOwner
import ai.nomi.feature.chat.ApprovalState
import ai.nomi.feature.chat.views.ChatView
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp

// The message blocks drawn with the design system's chat cards (`frontend/src/lib/components/ds/chat`), as the web
// draws them: approvals, to-do lists, file changes as diffs, thinking, follow-ups, link previews and shell output.

private val NoteGap = 6.dp
private const val REMEMBER_LABEL = "Remember this decision for next time"
private const val EXPIRED = "This request expired. Ask again and the agent will ask for your OK again."
/** A file diffed past this many lines shows the first ones only, like the web's collapsed diff. */
private const val MAX_DIFF_LINES = 400
/** Past this many line pairs the diff isn't worked out line by line: the old lines go, the new ones come. */
private const val MAX_DIFF_CELLS = 160_000L

/** `ApprovalCard`: what the agent wants to do and with which tool; approve (remembered or not) or deny. */
@Composable
internal fun ApprovalBlock(block: ContentBlock.Approval, agent: String?, state: ApprovalState, onResolve: (approve: Boolean, remember: Boolean) -> Unit) {
    val busy = state.sending || state.sent
    val status = when (block.status) {
        ApprovalStatus.Pending -> DsApprovalStatus.Pending
        ApprovalStatus.Approved -> DsApprovalStatus.Approved
        ApprovalStatus.Denied, ApprovalStatus.Cancelled -> DsApprovalStatus.Rejected
    }
    val open = status == DsApprovalStatus.Pending && !state.expired
    Column(verticalArrangement = Arrangement.spacedBy(NoteGap)) {
        DsApprovalCard(
            agent = Crew.forAgentType(agent),
            title = block.description,
            status = status,
            fields = listOf(KeyValue("Tool", block.toolName)),
            rememberLabel = REMEMBER_LABEL,
            rejectLabel = "Deny",
            outcome = when (block.status) {
                ApprovalStatus.Approved -> "Approved"
                ApprovalStatus.Denied -> "Denied"
                ApprovalStatus.Cancelled -> "Stopped before you decided"
                ApprovalStatus.Pending -> null
            },
            onApprove = if (open && !busy) ({ remember -> onResolve(true, remember) }) else null,
            onReject = if (open && !busy) ({ onResolve(false, false) }) else null,
        )
        val note = state.error ?: if (block.status == ApprovalStatus.Pending && state.expired) EXPIRED else null
        if (note != null) {
            Text(note, style = Ds.caption, color = if (state.error != null) Ds.danger else Ds.text2, modifier = Modifier.semantics { liveRegion = LiveRegionMode.Polite })
        }
    }
}

/** The agent's to-do list as the design system's plan: each step, where it stands, and who owns it. */
@Composable
internal fun TodoBlock(block: ContentBlock.TodoList, agent: String?) {
    val crew = Crew.forAgentType(agent)
    DsPlanCard(
        subject = "${agent ?: crew.displayName}’s to-do",
        steps = block.items.map { item ->
            PlanStep(
                item.text,
                when (item.status) {
                    TodoStatus.Done -> StepState.Done
                    TodoStatus.InProgress -> StepState.Running
                    TodoStatus.Pending -> StepState.Waiting
                },
                StepOwner(crew, crew.displayName),
            )
        },
    )
}

/** A file the agent created, changed or deleted, as the design system's diff (with its +/− counts). */
@Composable
internal fun FileChangeBlock(block: ContentBlock.FileChange) {
    val hunks = remember(block) { fileHunks(block) }
    Diff(block.path, hunks)
}

private fun lines(text: String?): List<String> = if (text.isNullOrEmpty()) emptyList() else text.removeSuffix("\n").split('\n')

/** One hunk for the whole file: a created file all added, a deleted one all removed, a change diffed by line. */
internal fun fileHunks(block: ContentBlock.FileChange): List<DiffHunk> {
    val before = if (block.action == FileAction.Created) emptyList() else lines(block.previousContent)
    val after = if (block.action == FileAction.Deleted) emptyList() else lines(block.content)
    val diff = lineDiff(before, after).take(MAX_DIFF_LINES)
    if (diff.isEmpty()) return emptyList()
    return listOf(DiffHunk("@@ -1,${before.size} +1,${after.size} @@", diff))
}

/** Lines of [a] → [b], by longest common subsequence, numbered on both sides. */
internal fun lineDiff(a: List<String>, b: List<String>): List<DiffLine> {
    val out = mutableListOf<DiffLine>()
    if (a.size.toLong() * b.size > MAX_DIFF_CELLS) {
        a.forEachIndexed { i, text -> out += DiffLine(DiffLineKind.Del, text, i + 1, null) }
        b.forEachIndexed { j, text -> out += DiffLine(DiffLineKind.Add, text, null, j + 1) }
        return out
    }
    val lcs = Array(a.size + 1) { IntArray(b.size + 1) }
    for (i in a.indices.reversed()) for (j in b.indices.reversed()) {
        lcs[i][j] = if (a[i] == b[j]) lcs[i + 1][j + 1] + 1 else maxOf(lcs[i + 1][j], lcs[i][j + 1])
    }
    var i = 0
    var j = 0
    while (i < a.size || j < b.size) {
        when {
            i < a.size && j < b.size && a[i] == b[j] -> { out += DiffLine(DiffLineKind.Context, a[i], i + 1, j + 1); i++; j++ }
            j < b.size && (i == a.size || lcs[i][j + 1] >= lcs[i + 1][j]) -> { out += DiffLine(DiffLineKind.Add, b[j], null, j + 1); j++ }
            else -> { out += DiffLine(DiffLineKind.Del, a[i], i + 1, null); i++ }
        }
    }
    return out
}

/** The model's thinking, folded under "Thinking" (the design system's Reasoning). */
@Composable
internal fun ReasoningBlock(text: String) {
    var open by rememberSaveable(text) { mutableStateOf(false) }
    Reasoning("Thinking", text, open, onToggle = { open = !open })
}

/** Suggested next messages as the design system's follow-ups; one press sends it, then they go. */
@Composable
internal fun FollowUpsBlock(view: ChatView.Suggestions, answered: Boolean, onSend: ((String) -> Unit)?) {
    var sent by rememberSaveable(view) { mutableStateOf<String?>(null) }
    if (answered || sent != null || onSend == null) {
        sent?.let { Text(it, style = Ds.caption, color = Ds.text2) }
        return
    }
    FollowUps(view.items, onPick = { item -> sent = item; onSend(item) })
}

/** A link with its site, title and description; opens in the browser. */
@Composable
internal fun LinkBlock(view: ChatView.LinkPreview) {
    val uri = LocalUriHandler.current
    val host = remember(view.url) { view.url.substringAfter("://").substringBefore('/').removePrefix("www.").ifEmpty { view.url } }
    LinkPreview(view.site ?: host, view.title ?: view.url, onOpen = { runCatching { uri.openUri(view.url) } }, description = view.description)
}

/** A command the agent ran and what it printed, in the design system's terminal; a non-zero exit is marked failed. */
@Composable
internal fun ShellBlock(view: ChatView.Shell) {
    val failed = view.exitCode != null && view.exitCode != 0
    val lines = buildList {
        add(TermLine(TermLineKind.Command, view.command))
        view.output?.removeSuffix("\n")?.split('\n')?.forEach { add(TermLine(TermLineKind.Out, it)) }
        view.exitCode?.let { code -> add(TermLine(if (code == 0) TermLineKind.Ok else TermLineKind.Fail, "exit $code")) }
    }
    Terminal(
        run = view.command,
        lines = lines,
        note = view.cwd ?: "read-only",
        failureTitle = if (failed) "Exited with ${view.exitCode}" else null,
    )
}
