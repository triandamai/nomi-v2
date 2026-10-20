package ai.nomi.data

import ai.nomi.network.CrewMemberDto
import ai.nomi.network.HomeApi
import ai.nomi.network.PreferencesDto
import ai.nomi.network.ProfileDto
import ai.nomi.network.UpdatePreferencesDto
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals

class FakeHomeApi : HomeApi {
    var storedPrefs = PreferencesDto(
        theme = "system",
        accentColor = "canopy",
        timezone = "UTC",
        homeLayout = "stage",
        drawerPins = listOf("money", "reminders", "memory", "notifications"),
        drawerRecents = true,
        drawerRecentsLimit = 10,
    )
    val updateRequests = mutableListOf<UpdatePreferencesDto>()

    override suspend fun profile(): ProfileDto = ProfileDto("Ana Lima", "ana", "ana@nomi.test", null)

    override suspend fun crew(): List<CrewMemberDto> = emptyList()

    override suspend fun preferences(): PreferencesDto = storedPrefs

    override suspend fun updatePreferences(request: UpdatePreferencesDto): PreferencesDto {
        updateRequests.add(request)
        storedPrefs = storedPrefs.copy(
            theme = request.theme ?: storedPrefs.theme,
            accentColor = request.accentColor ?: storedPrefs.accentColor,
            timezone = request.timezone ?: storedPrefs.timezone,
            homeLayout = request.homeLayout ?: storedPrefs.homeLayout,
            drawerPins = request.drawerPins ?: storedPrefs.drawerPins,
            drawerRecents = request.drawerRecents ?: storedPrefs.drawerRecents,
            drawerRecentsLimit = request.drawerRecentsLimit ?: storedPrefs.drawerRecentsLimit,
        )
        return storedPrefs
    }
}

@OptIn(ExperimentalCoroutinesApi::class)
class AppearanceRepositoryTest {
    @Test
    fun refreshLoadsHomeLayoutAndDrawerPreferences() = runTest {
        val fakeApi = FakeHomeApi()
        fakeApi.storedPrefs = fakeApi.storedPrefs.copy(
            theme = "dark",
            accentColor = "coral-reef",
            homeLayout = "classic",
            drawerRecents = false,
            drawerRecentsLimit = 20,
        )
        val session = backgroundScope.openSession()
        val repo = AppearanceRepository(fakeApi, session)

        repo.refresh()

        assertEquals("dark", repo.theme.value)
        assertEquals("coral-reef", repo.accentColor.value)
        assertEquals("classic", repo.homeLayout.value)
        assertEquals(false, repo.drawerRecents.value)
        assertEquals(20, repo.drawerRecentsLimit.value)
        assertEquals("Ana Lima", repo.profile.value?.displayName)
    }

    @Test
    fun selectHomeLayoutUpdatesStateAndSaves() = runTest {
        val fakeApi = FakeHomeApi()
        val session = openSession()
        try {
            val repo = AppearanceRepository(fakeApi, session, log = { println("LOG: $it") })

            repo.selectHomeLayout("today")
            assertEquals("today", repo.homeLayout.value)

            advanceUntilIdle()
            assertEquals(listOf("today"), fakeApi.updateRequests.map { it.homeLayout })
        } finally {
            session.close()
        }
    }

    @Test
    fun selectTimezoneUpdatesStateAndSaves() = runTest {
        val fakeApi = FakeHomeApi()
        val session = openSession()
        try {
            val repo = AppearanceRepository(fakeApi, session)
            repo.selectTimezone("Asia/Jakarta")
            assertEquals("Asia/Jakarta", repo.timezone.value)

            advanceUntilIdle()
            assertEquals("Asia/Jakarta", fakeApi.storedPrefs.timezone)
        } finally {
            session.close()
        }
    }

    @Test
    fun updateDrawerPinsAndTogglePinUpdatesStateAndSaves() = runTest {
        val fakeApi = FakeHomeApi()
        val session = openSession()
        try {
            val repo = AppearanceRepository(fakeApi, session)
            repo.updateDrawerPins(listOf("money", "reminders"))
            assertEquals(listOf("money", "reminders"), repo.drawerPins.value)

            advanceUntilIdle()
            assertEquals(listOf("money", "reminders"), fakeApi.storedPrefs.drawerPins)

            repo.togglePin("memory")
            assertEquals(listOf("money", "reminders", "memory"), repo.drawerPins.value)

            advanceUntilIdle()
            assertEquals(listOf("money", "reminders", "memory"), fakeApi.storedPrefs.drawerPins)

            repo.togglePin("money")
            assertEquals(listOf("reminders", "memory"), repo.drawerPins.value)
        } finally {
            session.close()
        }
    }
}



