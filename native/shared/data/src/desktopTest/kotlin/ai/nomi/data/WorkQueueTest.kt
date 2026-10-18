package ai.nomi.data

import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals

class WorkQueueTest {
    @Test
    fun an_item_throwing_an_Error_does_not_stop_the_queue() = runTest {
        val ran = mutableListOf<String>()
        val queue = WorkQueue(backgroundScope) {}
        queue.submit { TODO("not wired yet") }
        queue.submit { ran += "after" }
        runCurrent()
        assertEquals(listOf("after"), ran)
    }
}
