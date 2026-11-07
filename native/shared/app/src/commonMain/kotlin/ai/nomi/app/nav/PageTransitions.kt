package ai.nomi.app.nav

import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.ContentTransform
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.core.CubicBezierEasing
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.animation.togetherWith
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.navigation3.scene.Scene

// The web's page transitions (`material3.css`, "Page transitions"): the page area moves, the rest holds still.
// Forward, the old page eases back and fades while the new one rises in on the expressive spatial curve; back runs
// the other way. Reduced motion swaps pages at once.

private const val OUT_MS = 160
private const val IN_MS = 420
private const val IN_DELAY_MS = 60
private const val OUT_SCALE = 0.99f
private val OutEasing = CubicBezierEasing(0.3f, 0f, 0.8f, 0.15f)
private val InEasing = CubicBezierEasing(0.05f, 0.7f, 0.1f, 1f)

/** `nomi-page-out`: up 8. `nomi-page-in`: from 16 below. */
private val ForwardOut = (-8).dp
private val ForwardIn = 16.dp
/** `nomi-page-out-back`: down 8. `nomi-page-in-back`: from 12 above. */
private val BackOut = 8.dp
private val BackIn = (-12).dp

private fun Density.px(dp: Dp) = dp.roundToPx()

private fun pageIn(density: Density, from: Dp): EnterTransition =
    fadeIn(tween(IN_MS, IN_DELAY_MS, InEasing)) +
        slideInVertically(tween(IN_MS, IN_DELAY_MS, InEasing)) { density.px(from) }

private fun pageOut(density: Density, to: Dp): ExitTransition =
    fadeOut(tween(OUT_MS, easing = OutEasing)) +
        slideOutVertically(tween(OUT_MS, easing = OutEasing)) { density.px(to) } +
        scaleOut(tween(OUT_MS, easing = OutEasing), targetScale = OUT_SCALE)

/** The three specs [androidx.navigation3.ui.NavDisplay] takes, for the current density and motion setting. */
class PageTransitions(private val density: Density, private val reducedMotion: Boolean) {
    private fun transform(back: Boolean): ContentTransform = when {
        reducedMotion -> EnterTransition.None togetherWith ExitTransition.None
        back -> pageIn(density, BackIn) togetherWith pageOut(density, BackOut)
        else -> pageIn(density, ForwardIn) togetherWith pageOut(density, ForwardOut)
    }

    fun <T : Any> forward(): AnimatedContentTransitionScope<Scene<T>>.() -> ContentTransform = { transform(back = false) }

    fun <T : Any> back(): AnimatedContentTransitionScope<Scene<T>>.() -> ContentTransform = { transform(back = true) }

    fun <T : Any> predictiveBack(): AnimatedContentTransitionScope<Scene<T>>.(Int) -> ContentTransform = { transform(back = true) }
}
