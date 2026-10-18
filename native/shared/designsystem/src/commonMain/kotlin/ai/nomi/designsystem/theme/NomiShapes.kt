package ai.nomi.designsystem.theme

import ai.nomi.designsystem.generated.CornerSpec
import ai.nomi.designsystem.generated.NomiCorners
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Shapes
import androidx.compose.ui.unit.dp

/** Chat bubble shapes: the tail corner is tight, the rest are round. */
class NomiBubbleShapes(val start: RoundedCornerShape, val end: RoundedCornerShape)

fun CornerSpec.toShape() = RoundedCornerShape(
    topStart = topStart.dp,
    topEnd = topEnd.dp,
    bottomEnd = bottomEnd.dp,
    bottomStart = bottomStart.dp,
)

fun nomiShapes(): Shapes = Shapes(
    extraSmall = RoundedCornerShape(NomiCorners.extraSmall.dp),
    small = RoundedCornerShape(NomiCorners.small.dp),
    medium = RoundedCornerShape(NomiCorners.medium.dp),
    large = RoundedCornerShape(NomiCorners.large.dp),
    largeIncreased = RoundedCornerShape(NomiCorners.largeIncreased.dp),
    extraLarge = RoundedCornerShape(NomiCorners.extraLarge.dp),
    extraLargeIncreased = RoundedCornerShape(NomiCorners.extraLargeIncreased.dp),
    extraExtraLarge = RoundedCornerShape(NomiCorners.extraExtraLarge.dp),
)

fun nomiBubbleShapes() = NomiBubbleShapes(NomiCorners.bubbleStart.toShape(), NomiCorners.bubbleEnd.toShape())
