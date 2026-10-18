package ai.nomi.feature.settings

import ai.nomi.designsystem.component.NomiButton
import ai.nomi.designsystem.component.NomiButtonVariant
import ai.nomi.designsystem.layout.NomiPage
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag

const val SETTINGS_SCREEN_TAG = "settings-screen"

@Composable
fun SettingsScreen(viewModel: SettingsViewModel, modifier: Modifier = Modifier) {
    NomiPage(title = "Settings", modifier = modifier.testTag(SETTINGS_SCREEN_TAG)) {
        NomiButton(onClick = viewModel::signOut, variant = NomiButtonVariant.Outlined) { Text("Sign out") }
    }
}
