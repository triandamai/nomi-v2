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

    val theme: StateFlow<String> = mutableTheme.asStateFlow()
    val accentColor: StateFlow<String> = mutableAccentColor.asStateFlow()

    /** Maps backend `accent_color` (`"canopy"` for the default green) to the generated `NomiPalettes` key. */
    fun paletteKey(accent: String = mutableAccentColor.value): String =
        if (accent == DEFAULT_ACCENT_COLOR) CANOPY_PALETTE_KEY else accent

    suspend fun refresh() {
        try {
            val prefs = home.preferences()
            mutableTheme.value = prefs.theme
            mutableAccentColor.value = prefs.accentColor
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

    private fun save(request: UpdatePreferencesDto) {
        session.scope.launch {
            try {
                val saved = home.updatePreferences(request)
                mutableTheme.value = saved.theme
                mutableAccentColor.value = saved.accentColor
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                log("appearance: save failed: $e")
            }
        }
    }
}
