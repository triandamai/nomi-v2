package ai.nomi.designsystem.component

import kotlin.test.Test
import kotlin.test.assertEquals

class SendButtonTest {
    @Test fun emptyIdleIsDisabled() = assertEquals(SendButtonState.Disabled, sendButtonState(text = "", sending = false))
    @Test fun blankIdleIsDisabled() = assertEquals(SendButtonState.Disabled, sendButtonState(text = "  \n", sending = false))
    @Test fun typedIdleIsReady() = assertEquals(SendButtonState.Ready, sendButtonState(text = "hi", sending = false))
    @Test fun typedSendingIsSending() = assertEquals(SendButtonState.Sending, sendButtonState(text = "hi", sending = true))
    @Test fun emptySendingIsSending() = assertEquals(SendButtonState.Sending, sendButtonState(text = "", sending = true))
}
