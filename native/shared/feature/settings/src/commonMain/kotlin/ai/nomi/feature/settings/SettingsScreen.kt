package ai.nomi.feature.settings

import ai.nomi.designsystem.component.NomiListPanel
import ai.nomi.designsystem.component.NomiRow
import ai.nomi.designsystem.icon.NomiGlyph
import ai.nomi.designsystem.icon.NomiIcon
import ai.nomi.designsystem.icon.NomiIcons
import ai.nomi.designsystem.layout.NomiPage
import ai.nomi.designsystem.theme.NomiTheme
import ai.nomi.network.ProfileDto
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle

const val SETTINGS_SCREEN_TAG = "settings-screen"

private val IconBadgeSize = 36.dp

@Composable
fun SettingsScreen(viewModel: SettingsViewModel, modifier: Modifier = Modifier) {
    val profile by viewModel.profile.collectAsStateWithLifecycle()
    SettingsContent(profile, onSignOut = viewModel::signOut, modifier = modifier)
}

/** The account page: who is signed in, then signing out — rows in panels, like the web's account menu. */
@Composable
fun SettingsContent(profile: ProfileDto?, onSignOut: () -> Unit, modifier: Modifier = Modifier) {
    NomiPage(title = "Settings", modifier = modifier.testTag(SETTINGS_SCREEN_TAG)) {
        val name = profile?.displayName?.takeIf { it.isNotBlank() } ?: profile?.username
        NomiListPanel {
            NomiRow(
                title = name ?: profile?.email ?: "Your account",
                line = if (name != null) profile?.email else null,
                onClick = null,
                leading = { IconBadge(NomiIcons.Person, MaterialTheme.colorScheme.primaryContainer, MaterialTheme.colorScheme.onPrimaryContainer) },
            )
        }
        NomiListPanel {
            NomiRow(
                title = "Log out",
                onClick = onSignOut,
                leading = { IconBadge(NomiIcons.Logout, MaterialTheme.colorScheme.errorContainer, MaterialTheme.colorScheme.onErrorContainer) },
            )
        }
        Text("Nomi for ${platformName()}", style = NomiTheme.metaStyle, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

@Composable
private fun IconBadge(glyph: NomiGlyph, container: Color, ink: Color) {
    Box(Modifier.size(IconBadgeSize).clip(CircleShape).background(container), contentAlignment = Alignment.Center) {
        NomiIcon(glyph, contentDescription = null, tint = ink)
    }
}
