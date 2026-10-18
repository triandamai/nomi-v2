package ai.nomi.designsystem.component

import androidx.compose.ui.input.key.Key
import kotlin.test.Test
import kotlin.test.assertEquals

class ComposerKeyTest {
    @Test fun desktopEnterSends() =
        assertEquals(ComposerKeyAction.Send, composerKeyAction(Key.Enter, shift = false, platform = ComposerPlatform.Desktop))

    @Test fun desktopShiftEnterIsNewline() =
        assertEquals(ComposerKeyAction.Newline, composerKeyAction(Key.Enter, shift = true, platform = ComposerPlatform.Desktop))

    @Test fun desktopNumpadEnterSends() =
        assertEquals(ComposerKeyAction.Send, composerKeyAction(Key.NumPadEnter, shift = false, platform = ComposerPlatform.Desktop))

    @Test fun desktopOtherKeysPass() =
        assertEquals(ComposerKeyAction.Ignore, composerKeyAction(Key.A, shift = false, platform = ComposerPlatform.Desktop))

    @Test fun mobileEnterIsLeftToTheImeAction() {
        assertEquals(ComposerKeyAction.Ignore, composerKeyAction(Key.Enter, shift = false, platform = ComposerPlatform.Mobile))
        assertEquals(ComposerKeyAction.Ignore, composerKeyAction(Key.Enter, shift = true, platform = ComposerPlatform.Mobile))
    }
}
