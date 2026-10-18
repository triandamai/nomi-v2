package ai.nomi.designsystem.component

import androidx.compose.foundation.layout.Box
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.SplitButtonDefaults
import androidx.compose.material3.SplitButtonLayout
import androidx.compose.material3.Text
import androidx.compose.material3.LocalContentColor
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics

/**
 * A primary action with a menu chevron beside it: M3's SplitButtonLayout. The menu itself is M3's DropdownMenu,
 * passed as [menu] and shown while [expanded].
 */
@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun NomiSplitButton(
    label: String,
    onClick: () -> Unit,
    expanded: Boolean,
    onExpandedChange: (Boolean) -> Unit,
    menuLabel: String,
    modifier: Modifier = Modifier,
    menu: @Composable () -> Unit,
) {
    Box(modifier) {
        SplitButtonLayout(
            leadingButton = {
                SplitButtonDefaults.LeadingButton(onClick = onClick) { Text(label) }
            },
            trailingButton = {
                SplitButtonDefaults.TrailingButton(
                    checked = expanded,
                    onCheckedChange = onExpandedChange,
                    modifier = Modifier.semantics { contentDescription = menuLabel },
                ) { ChevronDownGlyph(LocalContentColor.current, SplitButtonDefaults.TrailingIconSize) }
            },
        )
        menu()
    }
}
