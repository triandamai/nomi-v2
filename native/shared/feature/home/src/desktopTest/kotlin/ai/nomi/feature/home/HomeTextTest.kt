package ai.nomi.feature.home

import ai.nomi.network.CrewMemberDto
import kotlin.test.Test
import kotlin.test.assertEquals

class HomeTextTest {
    @Test
    fun greetingFollowsTheHourAndUsesTheFirstName() {
        assertEquals("Still up, Ana.", greeting(hour = 3, displayName = "Ana Lima"))
        assertEquals("Morning, Ana.", greeting(hour = 9, displayName = "Ana Lima"))
        assertEquals("Afternoon.", greeting(hour = 14, displayName = null))
        assertEquals("Evening.", greeting(hour = 20, displayName = "  "))
    }

    @Test
    fun subtitleSaysWhereTheCrewIsWorking() {
        assertEquals("What should we get done?", subtitle(workingChats = 0))
        assertEquals("Your crew is working in 1 chat.", subtitle(workingChats = 1))
        assertEquals("Your crew is working in 3 chats.", subtitle(workingChats = 3))
    }

    @Test
    fun crewIsBusiestFirstKeepingRosterOrderForTies() {
        fun m(type: String, state: String) = CrewMemberDto(type, type, "", false, null, null, null, state, "")
        val roster = listOf(m("chitchat", "idle"), m("money", "done"), m("coding", "working"), m("planning", "waiting"), m("files", "idle"))
        assertEquals(listOf("coding", "planning", "money", "chitchat", "files"), crewByActivity(roster).map { it.agentType })
    }
}
