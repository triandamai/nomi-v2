package ai.nomi.designsystem.shape

import ai.nomi.designsystem.generated.PolarSpec
import ai.nomi.designsystem.generated.SHAPE_CENTER
import androidx.graphics.shapes.RoundedPolygon

/** The outline of this shape as a [RoundedPolygon] (for M3's LoadingIndicator morphs), in the 48-box around its centre. */
fun PolarSpec.toRoundedPolygon(): RoundedPolygon =
    RoundedPolygon(polarPoints(this), centerX = SHAPE_CENTER, centerY = SHAPE_CENTER)
