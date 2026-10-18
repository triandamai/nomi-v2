package ai.nomi.designsystem.theme

import ai.nomi.designsystem.generated.MotionSpec
import androidx.compose.animation.core.CubicBezierEasing
import androidx.compose.animation.core.tween

/** Nomi's generated cubic-bezier motion tokens as Compose animation specs. */
fun MotionSpec.easing() = CubicBezierEasing(x1, y1, x2, y2)

fun <T> MotionSpec.toTween() = tween<T>(durationMillis = durationMs, easing = easing())
