package ai.nomi.data

import ai.nomi.network.HomeApi
import ai.nomi.network.UpdatePreferencesDto
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

const val THEME_LIGHT = "light"
const val THEME_DARK = "dark"
const val THEME_SYSTEM = "system"
const val DEFAULT_ACCENT_COLOR = "canopy"
private const val CANOPY_PALETTE_KEY = "green"

/**
 * The signed-in person's appearance choices (`theme` mode and `accent_color` palette), shared between the root
 * `NomiTheme` wrapper and `SettingsScreen`. Picks apply immediately and save to `/api/preferences` in the background.
 */
class AppearanceRepository(
    private val home: HomeApi,
    private val session: SessionScope,
    private val log: (String) -> Unit = {},
) {
    private val mutableTheme = MutableStateFlow(THEME_SYSTEM)
    private val mutableAccentColor = MutableStateFlow(DEFAULT_ACCENT_COLOR)
    private val mutableHomeLayout = MutableStateFlow("stage")
    private val mutableDrawerPins = MutableStateFlow(listOf("money", "reminders", "memory", "notifications"))
    private val mutableDrawerRecents = MutableStateFlow(true)
    private val mutableDrawerRecentsLimit = MutableStateFlow(10)
    private val mutableTimezone = MutableStateFlow("UTC")
    private val mutableProfile = MutableStateFlow<ai.nomi.network.ProfileDto?>(null)

    val theme: StateFlow<String> = mutableTheme.asStateFlow()
    val accentColor: StateFlow<String> = mutableAccentColor.asStateFlow()
    val homeLayout: StateFlow<String> = mutableHomeLayout.asStateFlow()
    val drawerPins: StateFlow<List<String>> = mutableDrawerPins.asStateFlow()
    val drawerRecents: StateFlow<Boolean> = mutableDrawerRecents.asStateFlow()
    val drawerRecentsLimit: StateFlow<Int> = mutableDrawerRecentsLimit.asStateFlow()
    val timezone: StateFlow<String> = mutableTimezone.asStateFlow()
    val profile: StateFlow<ai.nomi.network.ProfileDto?> = mutableProfile.asStateFlow()

    /** Maps backend `accent_color` (`"canopy"` for the default green) to the generated `NomiPalettes` key. */
    fun paletteKey(accent: String = mutableAccentColor.value): String =
        if (accent == DEFAULT_ACCENT_COLOR) CANOPY_PALETTE_KEY else accent

    suspend fun refresh() {
        try {
            mutableProfile.value = home.profile()
            val prefs = home.preferences()
            mutableTheme.value = prefs.theme
            mutableAccentColor.value = prefs.accentColor
            mutableHomeLayout.value = prefs.homeLayout ?: "stage"
            prefs.drawerPins?.let { mutableDrawerPins.value = it }
            mutableDrawerRecents.value = prefs.drawerRecents
            mutableDrawerRecentsLimit.value = prefs.drawerRecentsLimit
            mutableTimezone.value = prefs.timezone
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            log("appearance: refresh failed: $e")
        }
    }

    fun selectTheme(theme: String) {
        if (mutableTheme.value == theme) return
        mutableTheme.value = theme
        save(UpdatePreferencesDto(theme = theme))
    }

    fun selectAccentColor(accentColor: String) {
        if (mutableAccentColor.value == accentColor) return
        mutableAccentColor.value = accentColor
        save(UpdatePreferencesDto(accentColor = accentColor))
    }

    fun selectHomeLayout(layout: String) {
        if (mutableHomeLayout.value == layout) return
        mutableHomeLayout.value = layout
        save(UpdatePreferencesDto(homeLayout = layout))
    }

    fun toggleDrawerRecents(enabled: Boolean) {
        if (mutableDrawerRecents.value == enabled) return
        mutableDrawerRecents.value = enabled
        save(UpdatePreferencesDto(drawerRecents = enabled))
    }

    fun selectDrawerRecentsLimit(limit: Int) {
        if (mutableDrawerRecentsLimit.value == limit) return
        mutableDrawerRecentsLimit.value = limit
        save(UpdatePreferencesDto(drawerRecentsLimit = limit))
    }

    fun selectTimezone(timezone: String) {
        if (mutableTimezone.value == timezone) return
        mutableTimezone.value = timezone
        save(UpdatePreferencesDto(timezone = timezone))
    }

    fun updateDrawerPins(pins: List<String>) {
        if (mutableDrawerPins.value == pins) return
        mutableDrawerPins.value = pins
        save(UpdatePreferencesDto(drawerPins = pins))
    }

    fun togglePin(id: String) {
        val current = mutableDrawerPins.value
        val next = if (id in current) current - id else current + id
        updateDrawerPins(next)
    }

    private fun save(request: UpdatePreferencesDto) {
        session.scope.launch {
            try {
                val saved = home.updatePreferences(request)

                mutableTheme.value = saved.theme
                mutableAccentColor.value = saved.accentColor
                saved.homeLayout?.let { mutableHomeLayout.value = it }
                saved.drawerPins?.let { mutableDrawerPins.value = it }
                mutableDrawerRecents.value = saved.drawerRecents
                mutableDrawerRecentsLimit.value = saved.drawerRecentsLimit
                mutableTimezone.value = saved.timezone
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                log("appearance: save failed: $e")
            }
        }
    }
}
