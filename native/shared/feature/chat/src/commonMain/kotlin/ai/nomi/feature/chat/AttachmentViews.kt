package ai.nomi.feature.chat

import ai.nomi.designsystem.icon.NomiGlyph
import ai.nomi.designsystem.icon.NomiIcon
import ai.nomi.designsystem.icon.NomiIcons
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import org.jetbrains.compose.resources.decodeToImageBitmap

/** `.images`: one image up to 360px wide, several in a two-column grid. */
private val ImagesMaxWidth = 280.dp
private val ImageGap = 4.dp
private const val GRID_COLUMNS = 2
private const val SINGLE_IMAGE_RATIO = 4f / 3f
private const val GRID_IMAGE_RATIO = 1f
private val CardMaxWidth = 280.dp
private val CardPaddingH = 12.dp
private val CardPaddingV = 10.dp
private val CardGap = 10.dp
private val CardIconBox = 36.dp
private val CardIconSize = 20.dp
private val FallbackIconSize = 22.dp
private val PreviewMaxHeight = 240.dp
private val PreviewPadding = 12.dp
private val SectionGap = 6.dp
private val ViewerPadding = 8.dp
private const val VIEWER_SCRIM_ALPHA = 0.92f

/**
 * The files of a sent message (`MessageAttachments.svelte`): images as previews (tap for the whole image), everything
 * else as a card with its kind, name and size. Older messages' inline text files open onto their text.
 */
@Composable
fun MessageAttachments(files: List<Attachment>, modifier: Modifier = Modifier) {
    val images = files.filter { it.id != null && it.kind == AttachmentKind.Image }
    val others = files.filterNot { it.id != null && it.kind == AttachmentKind.Image }
    var viewing by remember { mutableStateOf<Attachment?>(null) }
    Column(modifier, horizontalAlignment = Alignment.End, verticalArrangement = Arrangement.spacedBy(SectionGap)) {
        if (images.isNotEmpty()) ImageGrid(images, onOpen = { viewing = it })
        others.forEach { FileCard(it) }
    }
    viewing?.let { image -> ImageViewer(image, onClose = { viewing = null }) }
}

@Composable
private fun ImageGrid(images: List<Attachment>, onOpen: (Attachment) -> Unit) {
    val shape = MaterialTheme.shapes.large
    Column(
        Modifier.widthIn(max = ImagesMaxWidth).clip(shape),
        verticalArrangement = Arrangement.spacedBy(ImageGap),
    ) {
        if (images.size == 1) {
            AttachedImage(images.single(), SINGLE_IMAGE_RATIO, Modifier.fillMaxWidth(), onOpen)
        } else {
            images.chunked(GRID_COLUMNS).forEach { row ->
                Row(horizontalArrangement = Arrangement.spacedBy(ImageGap)) {
                    row.forEach { AttachedImage(it, GRID_IMAGE_RATIO, Modifier.weight(1f), onOpen) }
                    // An odd last image keeps its column's width.
                    repeat(GRID_COLUMNS - row.size) { Box(Modifier.weight(1f)) }
                }
            }
        }
    }
}

@Composable
private fun AttachedImage(image: Attachment, ratio: Float, modifier: Modifier, onOpen: (Attachment) -> Unit) {
    val bitmap = rememberPreview(image.id)
    val colors = MaterialTheme.colorScheme
    Box(
        modifier
            .aspectRatio(ratio)
            .background(colors.surfaceContainerHigh)
            .semantics { contentDescription = "Open ${image.name}" }
            .clickable(role = Role.Image, onClickLabel = "Open ${image.name}") { onOpen(image) },
        contentAlignment = Alignment.Center,
    ) {
        if (bitmap != null) {
            Image(bitmap, contentDescription = image.name, contentScale = ContentScale.Crop, modifier = Modifier.fillMaxSize())
        } else {
            Column(horizontalAlignment = Alignment.CenterHorizontally, modifier = Modifier.padding(CardPaddingH)) {
                NomiIcon(NomiIcons.Image, contentDescription = null, size = FallbackIconSize, tint = colors.onSurfaceVariant)
                Text(
                    image.name,
                    style = MaterialTheme.typography.labelSmall,
                    color = colors.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }
    }
}

/** The preview of the attachment [id], decoded; null while loading or when it can't be had. */
@Composable
private fun rememberPreview(id: String?): ImageBitmap? {
    val load = LocalAttachmentPreviews.current
    val bitmap by produceState<ImageBitmap?>(null, id) {
        val bytes = id?.let { load(it) } ?: return@produceState
        value = runCatching { bytes.decodeToImageBitmap() }.getOrNull()
    }
    return bitmap
}

@Composable
private fun FileCard(file: Attachment) {
    val colors = MaterialTheme.colorScheme
    val inline = file.text
    var open by rememberSaveable(file.name, file.id) { mutableStateOf(false) }
    val size = file.size ?: inline?.encodeToByteArray()?.size?.toLong()
    Column(Modifier.widthIn(max = CardMaxWidth), horizontalAlignment = Alignment.End) {
        Row(
            Modifier
                .clip(MaterialTheme.shapes.large)
                .background(colors.surfaceContainerLow)
                .then(
                    if (inline != null) {
                        Modifier.clickable(role = Role.Button, onClickLabel = if (open) "Hide ${file.name}" else "Show ${file.name}") { open = !open }
                    } else {
                        Modifier
                    },
                )
                .semantics(mergeDescendants = true) {}
                .padding(horizontal = CardPaddingH, vertical = CardPaddingV),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(CardGap),
        ) {
            Box(Modifier.size(CardIconBox).background(colors.secondaryContainer, CircleShape), contentAlignment = Alignment.Center) {
                NomiIcon(file.kind.glyph(), contentDescription = null, size = CardIconSize, tint = colors.onSecondaryContainer)
            }
            Column(Modifier.widthIn(max = CardMaxWidth - CardIconBox - CardGap - CardPaddingH * 2)) {
                Text(
                    file.name,
                    style = MaterialTheme.typography.bodyMedium.copy(fontWeight = FontWeight.SemiBold),
                    color = colors.onSurface,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                size?.let { Text(formatBytes(it), style = MaterialTheme.typography.labelSmall, color = colors.onSurfaceVariant) }
            }
        }
        if (open && inline != null) {
            Text(
                inline,
                style = MaterialTheme.typography.bodySmall.copy(fontFamily = FontFamily.Monospace),
                color = colors.onSurface,
                modifier = Modifier
                    .padding(top = SectionGap)
                    .widthIn(max = CardMaxWidth)
                    .heightIn(max = PreviewMaxHeight)
                    .clip(MaterialTheme.shapes.medium)
                    .background(colors.surfaceContainer)
                    .verticalScroll(rememberScrollState())
                    .horizontalScroll(rememberScrollState())
                    .padding(PreviewPadding),
            )
        }
    }
}

/** The whole image over the chat; its preview stands in until (and if) the full file is loaded. */
@Composable
private fun ImageViewer(image: Attachment, onClose: () -> Unit) {
    val bitmap = rememberPreview(image.id)
    Dialog(onDismissRequest = onClose, properties = DialogProperties(usePlatformDefaultWidth = false)) {
        Box(
            Modifier
                .fillMaxSize()
                .background(Color.Black.copy(alpha = VIEWER_SCRIM_ALPHA))
                .clickable(onClickLabel = "Close", onClick = onClose)
                .safeDrawingPadding(),
            contentAlignment = Alignment.Center,
        ) {
            if (bitmap != null) {
                Image(bitmap, contentDescription = image.name, contentScale = ContentScale.Fit, modifier = Modifier.fillMaxWidth())
            } else {
                Text(image.name, color = Color.White, style = MaterialTheme.typography.bodyLarge)
            }
            IconButton(onClick = onClose, modifier = Modifier.align(Alignment.TopEnd).padding(ViewerPadding)) {
                NomiIcon(NomiIcons.Close, contentDescription = "Close image", tint = Color.White)
            }
        }
    }
}

private fun AttachmentKind.glyph(): NomiGlyph = when (this) {
    AttachmentKind.Image -> NomiIcons.Image
    AttachmentKind.Video -> NomiIcons.Video
    AttachmentKind.Audio -> NomiIcons.Audio
    AttachmentKind.Voice -> NomiIcons.Voice
    AttachmentKind.Spreadsheet -> NomiIcons.Spreadsheet
    AttachmentKind.Presentation -> NomiIcons.Presentation
    AttachmentKind.Pdf -> NomiIcons.Pdf
    AttachmentKind.Text, AttachmentKind.Document, AttachmentKind.Other, AttachmentKind.InlineText -> NomiIcons.File
}
