package ai.nomi.designsystem.shape

import ai.nomi.designsystem.generated.NomiShapeSpecs
import ai.nomi.designsystem.generated.SHAPE_POINTS
import kotlin.test.Test
import kotlin.test.assertEquals

class PolarPolygonTest {
    @Test fun polygonHasOneCornerPerOutlinePoint() {
        val polygon = NomiShapeSpecs.all.getValue("cookie9").toRoundedPolygon()
        assertEquals(SHAPE_POINTS, polygon.cubics.size)
    }
}
