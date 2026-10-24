package ai.nomi.feature.chat.blocks

import kotlin.test.Test
import kotlin.test.assertEquals

class ContentBlockTest {
    @Test fun noJsonMeansNoBlocks() {
        assertEquals(emptyList(), parseContentBlocks(null))
        assertEquals(emptyList(), parseContentBlocks(""))
        assertEquals(emptyList(), parseContentBlocks("not json"))
        assertEquals(emptyList(), parseContentBlocks("""{"kind":"todo_list","items":[]}"""))
    }

    @Test fun approvalRequest() {
        val blocks = parseContentBlocks(
            """[{"kind":"approval_request","id":"a1","tool_name":"delete_file","description":"Delete src/old.js",
               "input":{"path":"src/old.js"},"status":"pending","decided_at":null}]""",
        )
        assertEquals(listOf(ContentBlock.Approval("a1", "delete_file", "Delete src/old.js", ApprovalStatus.Pending)), blocks)
    }

    @Test fun everyApprovalStatus() {
        ApprovalStatus.entries.forEach { status ->
            val block = parseContentBlocks("""[{"kind":"approval_request","id":"a","tool_name":"t","description":"d","status":"${status.wire}"}]""")
            assertEquals(status, (block.single() as ContentBlock.Approval).status)
        }
    }

    @Test fun approvalWithAnUnknownStatusFallsBack() {
        val blocks = parseContentBlocks("""[{"kind":"approval_request","id":"a","tool_name":"t","description":"d","status":"later"}]""")
        assertEquals(listOf(ContentBlock.Unknown("approval_request")), blocks)
    }

    @Test fun todoList() {
        val blocks = parseContentBlocks(
            """[{"kind":"todo_list","items":[
                 {"id":"1","text":"Read the brief","status":"done"},
                 {"id":"2","text":"Draft the page","status":"in_progress"},
                 {"id":"3","text":"Ship it","status":"pending"},
                 {"text":"No id","status":"someday"},
                 {"id":"5"}]}]""",
        )
        assertEquals(
            listOf(
                ContentBlock.TodoList(
                    listOf(
                        TodoItem("1", "Read the brief", TodoStatus.Done),
                        TodoItem("2", "Draft the page", TodoStatus.InProgress),
                        TodoItem("3", "Ship it", TodoStatus.Pending),
                        TodoItem("3", "No id", TodoStatus.Pending),
                    ),
                ),
            ),
            blocks,
        )
    }

    @Test fun plan() {
        val blocks = parseContentBlocks("""[{"kind":"plan","plan_id":"p1","agent_session_id":"as1","title":"Launch week","version":3}]""")
        assertEquals(listOf(ContentBlock.Plan("p1", "as1", "Launch week", 3)), blocks)
    }

    @Test fun fileBlocks() {
        val blocks = parseContentBlocks(
            """[
              {"kind":"file_write","project_id":"pr","path":"src/new.ts","content":"a\n","previous_content":null},
              {"kind":"file_write","project_id":"pr","path":"src/app.ts","content":"b\n","previous_content":"a\n"},
              {"kind":"file_delete","project_id":"pr","path":"src/old.ts","previous_content":"gone\n"},
              {"kind":"file_delete","project_id":"pr","path":"src/older.ts"}
            ]""",
        )
        assertEquals(
            listOf(
                ContentBlock.FileChange("src/new.ts", FileAction.Created, "a\n", null),
                ContentBlock.FileChange("src/app.ts", FileAction.Updated, "b\n", "a\n"),
                ContentBlock.FileChange("src/old.ts", FileAction.Deleted, null, "gone\n"),
                ContentBlock.FileChange("src/older.ts", FileAction.Deleted, null, null),
            ),
            blocks,
        )
    }

    @Test fun fileBatchBecomesOneChangePerFile() {
        val blocks = parseContentBlocks(
            """[{"kind":"file_writes","project_id":"pr","files":[
                 {"path":"a.md","content":"x","previous_content":null},
                 {"path":"b.md","content":"y","previous_content":"z"}]}]""",
        )
        assertEquals(
            listOf(
                ContentBlock.FileChange("a.md", FileAction.Created, "x", null),
                ContentBlock.FileChange("b.md", FileAction.Updated, "y", "z"),
            ),
            blocks,
        )
    }

    @Test fun reasoning() {
        assertEquals(listOf(ContentBlock.Reasoning("Weighing it up")), parseContentBlocks("""[{"kind":"reasoning","text":"Weighing it up"}]"""))
    }

    @Test fun unknownAndMalformedBlocksFallBack() {
        val blocks = parseContentBlocks(
            """[{"kind":"quota_notice","reason":"limit"},{"kind":"plan","title":"no ids"},{"no":"kind"},42,
               {"kind":"view","view":"hologram"},{"kind":"reasoning","text":"kept"}]""",
        )
        assertEquals(
            listOf(
                ContentBlock.Unknown("quota_notice"),
                ContentBlock.Unknown("plan"),
                ContentBlock.Unknown("unknown"),
                ContentBlock.Unknown("view"),
                ContentBlock.Reasoning("kept"),
            ),
            blocks,
        )
    }

    @Test fun viewBlockBecomesItsView() {
        val blocks = parseContentBlocks("""[{"kind":"view","view":"answer","label":"Left","value":1350000,"format":"money","tone":"positive"}]""")
        assertEquals(
            listOf(ContentBlock.View(ai.nomi.feature.chat.views.ChatView.Answer("Left", ai.nomi.feature.chat.views.Scalar.Num(1350000.0), ai.nomi.feature.chat.views.ValueFormat.Money, null, ai.nomi.feature.chat.views.Tone.Positive))),
            blocks,
        )
    }

    @Test fun olderTableBlockDrawsAsATableView() {
        val block = parseContentBlocks("""[{"kind":"table","variant":"data","columns":[{"key":"what","label":"What"}],"rows":[{"what":"Lunch"}]}]""").single()
        val view = (block as ContentBlock.View).view as ai.nomi.feature.chat.views.ChatView.Table
        assertEquals(listOf(mapOf("what" to ai.nomi.feature.chat.views.Scalar.Text("Lunch"))), view.rows)
    }

    @Test fun questionsBlock() {
        val block = parseContentBlocks(
            """[{"kind":"questions","intro":null,"questions":[{"question":"Which day?","options":[{"label":"Sat"},{"label":"Sun","detail":"rain"}],"multiple":true}]}]""",
        ).single() as ContentBlock.Questions
        assertEquals(null, block.block.intro)
        assertEquals(ai.nomi.feature.chat.views.Question("Which day?", listOf("Sat" to null, "Sun" to "rain"), multiple = true), block.block.questions.single())
    }

    @Test fun diffStatsCountsChangedLines() {
        assertEquals(DiffStats(added = 3, removed = 0), diffStats(null, "a\nb\nc\n"))
        assertEquals(DiffStats(added = 0, removed = 2), diffStats("a\nb", null))
        assertEquals(DiffStats(added = 1, removed = 1), diffStats("a\nb\nc\n", "a\nB\nc\n"))
        assertEquals(DiffStats(added = 1, removed = 0), diffStats("a\nc", "a\nb\nc"))
        assertEquals(DiffStats(added = 0, removed = 0), diffStats("same\n", "same\n"))
    }
}
