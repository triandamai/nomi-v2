package ai.nomi.designsystem.icon

/**
 * The Lucide line icons the design-system components draw (frontend `@lucide/svelte`), on the same
 * 24-unit grid and 2-unit stroke as [NomiIcons]. Only the ones the components use.
 */
object Lucide {
    private fun circle(cx: Float, cy: Float, r: Float) = "M${cx - r} ${cy}a$r $r 0 1 0 ${2 * r} 0a$r $r 0 1 0 ${-2 * r} 0"
    private fun dot(cx: Float, cy: Float) = circle(cx, cy, DOT_RADIUS)
    private fun rect(x: Float, y: Float, w: Float, h: Float, r: Float) =
        "M${x + r} ${y}h${w - 2 * r}a$r $r 0 0 1 $r ${r}v${h - 2 * r}a$r $r 0 0 1 ${-r} ${r}h${-(w - 2 * r)}a$r $r 0 0 1 ${-r} ${-r}v${-(h - 2 * r)}a$r $r 0 0 1 $r ${-r}z"
    private fun glyph(vararg d: String) = NomiGlyph(d.map { GlyphStroke(it) })

    private const val DOT_RADIUS = 1f
    private const val FILE = "M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7z"
    private const val FILE_FOLD = "M14 2v4a2 2 0 0 0 2 2h4"
    private const val PERSON = "M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"
    private const val SHIELD = "M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z"
    private const val WIFI = "M5 12.86a10 10 0 0 1 14 0"
    private const val BOOKMARK = "m19 21-7-4-7 4V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2v16z"
    private const val FOLDER = "M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"
    private const val BELL = "M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9"
    private const val BELL_CLAPPER = "M10.3 21a1.94 1.94 0 0 0 3.4 0"

    val Loader = glyph("M21 12a9 9 0 1 1-6.219-8.56")
    val Check = glyph("M20 6 9 17l-5-5")
    val X = glyph("M18 6 6 18", "m6 6 12 12")
    val Alert = glyph(circle(12f, 12f, 10f), "M12 8v4", "M12 16h.01")
    val Info = glyph(circle(12f, 12f, 10f), "M12 16v-4", "M12 8h.01")
    val Circle = glyph(circle(12f, 12f, 10f))
    val CircleDashed = NomiGlyph(listOf(GlyphStroke(circle(12f, 12f, 9f), dash = floatArrayOf(3.2f, 3.9f))))
    val Square = glyph(rect(3f, 3f, 18f, 18f, 2f))
    val Pencil = glyph(
        "M21.17 6.81a1 1 0 0 0-3.99-3.99L3.84 16.17a2 2 0 0 0-.5.83l-1.32 4.35a.5.5 0 0 0 .62.62l4.35-1.32a2 2 0 0 0 .83-.5z",
        "m15 5 4 4",
    )
    val Copy = glyph(rect(8f, 8f, 14f, 14f, 2f), "M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2")
    val Paperclip = glyph("m21.44 11.05-9.19 9.19a6 6 0 0 1-8.49-8.49l8.57-8.57A4 4 0 1 1 18 8.84l-8.59 8.57a2 2 0 0 1-2.83-2.83l8.49-8.48")
    val ArrowRight = glyph("M5 12h14", "m12 5 7 7-7 7")
    val ArrowLeft = glyph("m12 19-7-7 7-7", "M19 12H5")
    val ArrowUp = glyph("m5 12 7-7 7 7", "M12 19V5")
    val ArrowDown = glyph("M12 5v14", "m19 12-7 7-7-7")
    val ArrowUpRight = glyph("M7 7h10v10", "M7 17 17 7")
    val ChevronRight = glyph("m9 18 6-6-6-6")
    val ChevronLeft = glyph("m15 18-6-6 6-6")
    val ChevronDown = glyph("m6 9 6 6 6-6")
    val ChevronUp = glyph("m18 15-6-6-6 6")
    val ChevronsUpDown = glyph("m7 15 5 5 5-5", "m7 9 5-5 5 5")
    val CornerDownRight = glyph("m15 10 5 5-5 5", "M4 4v7a4 4 0 0 0 4 4h12")
    val CornerDownLeft = glyph("M20 4v7a4 4 0 0 1-4 4H4", "m9 10-5 5 5 5")
    val Plus = glyph("M5 12h14", "M12 5v14")
    val Minus = glyph("M5 12h14")
    val Search = glyph(circle(11f, 11f, 8f), "m21 21-4.3-4.3")
    val Mic = glyph(rect(9f, 2f, 6f, 13f, 3f), "M19 10v2a7 7 0 0 1-14 0v-2", "M12 19v3")
    val Send = glyph(
        "M14.54 21.69a.5.5 0 0 0 .94-.03l6.5-19a.5.5 0 0 0-.64-.63l-19 6.5a.5.5 0 0 0-.02.93l7.93 3.18a2 2 0 0 1 1.11 1.11z",
        "m21.85 2.15-10.94 10.94",
    )
    val Play = glyph("M6 3l14 9-14 9z")
    val Trash = glyph("M3 6h18", "M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6", "M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2", "M10 11v6", "M14 11v6")
    val Download = glyph("M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4", "m7 10 5 5 5-5", "M12 15V3")
    val Share = glyph(circle(18f, 5f, 3f), circle(6f, 12f, 3f), circle(18f, 19f, 3f), "m8.59 13.51 6.83 3.98", "m15.41 6.51-6.82 3.98")
    val Refresh = glyph(
        "M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8", "M21 3v5h-5",
        "M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16", "M8 16H3v5",
    )
    val RotateCcw = glyph("M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8", "M3 3v5h5")
    val ThumbsUp = NomiIcons.ThumbsUp
    val ThumbsDown = NomiIcons.ThumbsDown
    val Lightbulb = NomiIcons.Lightbulb
    val Sparkles = NomiIcons.Sparkle
    val Globe = NomiIcons.Globe
    val Database = NomiIcons.Database
    val Rocket = NomiIcons.Rocket
    val Clock = NomiIcons.Clock
    val Wallet = NomiIcons.Wallet
    val Image = NomiIcons.Image
    val Pin = NomiIcons.Pin
    val PinOff = glyph("M9 4h6l-1 5 3 3v2H7v-2l3-3z", "M12 14v6", "m2 2 20 20")
    val Lock = NomiIcons.Lock
    val Home = NomiIcons.Home
    val Settings = NomiIcons.Settings
    val Logout = NomiIcons.Logout
    val File = glyph(FILE, FILE_FOLD)
    val FileText = glyph(FILE, FILE_FOLD, "M10 9H8", "M16 13H8", "M16 17H8")
    val FileCode = glyph(FILE, FILE_FOLD, "m10 13-2 2 2 2", "m14 17 2-2-2-2")
    val FileDiff = glyph(FILE, FILE_FOLD, "M9 10h6", "M12 13V7", "M9 17h6")
    val FilePen = glyph("M12.5 22H18a2 2 0 0 0 2-2V7l-5-5H6a2 2 0 0 0-2 2v9.5", FILE_FOLD, "M13.38 15.62a1 1 0 0 0-3-3L5.37 17.63a2 2 0 0 0-.5.83l-.84 2.84a.5.5 0 0 0 .62.62l2.84-.84a2 2 0 0 0 .83-.5z")
    val Folder = glyph(FOLDER)
    val FolderInput = glyph("M2 9V5a2 2 0 0 1 2-2h3.9a2 2 0 0 1 1.69.9l.81 1.2a2 2 0 0 0 1.67.9H20a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2v-1", "M2 13h10", "m9 16 3-3-3-3")
    val Hourglass = glyph(
        "M5 22h14", "M5 2h14",
        "M17 22v-4.17a2 2 0 0 0-.59-1.41L12 12l-4.41 4.41A2 2 0 0 0 7 17.83V22",
        "M7 2v4.17a2 2 0 0 0 .59 1.41L12 12l4.41-4.41A2 2 0 0 0 17 6.17V2",
    )
    val Bookmark = glyph(BOOKMARK)
    val BookmarkPlus = glyph(BOOKMARK, "M12 7v6", "M15 10H9")
    val Bell = glyph(BELL, BELL_CLAPPER)
    val BellRing = glyph(BELL, BELL_CLAPPER, "M4 2C2.8 3.7 2 5.7 2 8", "M22 8c0-2.3-.8-4.3-2-6")
    val AlarmClock = glyph(circle(12f, 13f, 8f), "M12 9v4l2 2", "M5 3 2 6", "m22 6-3-3")
    val Shield = glyph(SHIELD)
    val ShieldCheck = glyph(SHIELD, "m9 12 2 2 4-4")
    val Eye = glyph("M2.06 12.35a1 1 0 0 1 0-.7 10.75 10.75 0 0 1 19.88 0 1 1 0 0 1 0 .7 10.75 10.75 0 0 1-19.88 0", circle(12f, 12f, 3f))
    val Unplug = glyph("m19 5 3-3", "m2 22 3-3", "M6.3 20.3a2.4 2.4 0 0 0 3.4 0L12 18l-6-6-2.3 2.3a2.4 2.4 0 0 0 0 3.4Z", "M7.5 13.5 10 11", "M10.5 16.5 13 14", "m12 6 6 6 2.3-2.3a2.4 2.4 0 0 0 0-3.4l-2.6-2.6a2.4 2.4 0 0 0-3.4 0Z")
    val Wifi = glyph("M12 20h.01", "M2 8.82a15 15 0 0 1 20 0", WIFI, "M8.5 16.43a5 5 0 0 1 7 0")
    val WifiOff = glyph("M12 20h.01", "M8.5 16.43a5 5 0 0 1 7 0", "M2 8.82a15 15 0 0 1 4.17-2.65", "M10.66 5c4.01-.36 8.14.9 11.34 3.76", "M16.85 11.25a10 10 0 0 1 2.22 1.68", "M5 12.86a10 10 0 0 1 5.17-2.69", "m2 2 20 20")
    val Activity = NomiIcons.Pulse
    val KeyRound = glyph(
        "M2.59 17.41A2 2 0 0 0 2 18.83V21a1 1 0 0 0 1 1h3a1 1 0 0 0 1-1v-1a1 1 0 0 1 1-1h1a1 1 0 0 0 1-1v-1a1 1 0 0 1 1-1h.17a2 2 0 0 0 1.41-.59l.81-.81a6.5 6.5 0 1 0-4-4z",
        circle(16.5f, 7.5f, 0.5f),
    )
    val Gauge = glyph("m12 14 4-4", "M3.34 19a10 10 0 1 1 17.32 0")
    val Split = glyph("M16 3h5v5", "M8 3H3v5", "M12 22v-8.3a4 4 0 0 0-1.17-2.87L3 3", "m15 9 6-6")
    val Grip = glyph(dot(9f, 12f), dot(9f, 5f), dot(9f, 19f), dot(15f, 12f), dot(15f, 5f), dot(15f, 19f))
    val Ellipsis = glyph(dot(12f, 12f), dot(19f, 12f), dot(5f, 12f))
    val Package = glyph("M11 21.73a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73z", "M12 22V12", "M3.29 7 12 12l8.71-5", "m7.5 4.27 9 5.15")
    val Basket = glyph("m15 11-1 9", "m19 11-4-7", "M2 11h20", "m3.5 11 1.6 7.4a2 2 0 0 0 2 1.6h9.8a2 2 0 0 0 2-1.6l1.7-7.4", "M4.5 15.5h15", "m5 11 4-7", "m9 11 1 9")
    val Table = glyph(rect(3f, 3f, 18f, 18f, 2f), "M3 9h18", "M9 3v18")
    val ListChecks = glyph("m3 17 2 2 4-4", "m3 7 2 2 4-4", "M13 6h8", "M13 12h8", "M13 18h8")
    val ListTree = glyph("M21 12h-8", "M21 6H8", "M21 18h-8", "M3 6v4c0 1.1.9 2 2 2h3", "M3 10v6c0 1.1.9 2 2 2h3")
    val List = glyph("M3 12h.01", "M3 18h.01", "M3 6h.01", "M8 12h13", "M8 18h13", "M8 6h13")
    val Brain = glyph(
        "M12 5a3 3 0 1 0-5.997.125 4 4 0 0 0-2.526 5.77 4 4 0 0 0 .556 6.588A4 4 0 1 0 12 18Z",
        "M12 5a3 3 0 1 1 5.997.125 4 4 0 0 1 2.526 5.77 4 4 0 0 1-.556 6.588A4 4 0 1 1 12 18Z",
        "M12 5v13",
    )
    val Maximize = glyph("M15 3h6v6", "M9 21H3v-6", "M21 3l-7 7", "M3 21l7-7")
    val Minimize = glyph("M4 14h6v6", "M20 10h-6V4", "M14 10l7-7", "M3 21l7-7")
    val LogIn = glyph("m10 17 5-5-5-5", "M15 12H3", "M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4")
    val Link = glyph("M9 17H7A5 5 0 0 1 7 7h2", "M15 7h2a5 5 0 1 1 0 10h-2", "M8 12h8")
    val Users = glyph(PERSON, circle(9f, 7f, 4f), "M22 21v-2a4 4 0 0 0-3-3.87", "M16 3.13a4 4 0 0 1 0 7.75")
    val User = glyph("M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2", circle(12f, 7f, 4f))
    val UserPlus = glyph(PERSON, circle(9f, 7f, 4f), "M19 8v6", "M22 11h-6")
    val HardDrive = glyph("M22 12H2", "M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z", "M6 16h.01", "M10 16h.01")
    val PanelLeft = glyph(rect(3f, 3f, 18f, 18f, 2f), "M9 3v18")
    val Message = glyph("M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z")
    val Smartphone = glyph(rect(5f, 2f, 14f, 20f, 2f), "M12 18h.01")
    val Monitor = glyph(rect(2f, 3f, 20f, 14f, 2f), "M8 21h8", "M12 17v4")
    val Terminal = glyph("m4 17 6-6-6-6", "M12 19h8")
    val Code = glyph("m16 18 6-6-6-6", "m8 6-6 6 6 6")
    val GitBranch = glyph("M6 3v12", circle(18f, 6f, 3f), circle(6f, 18f, 3f), "M18 9a9 9 0 0 1-9 9")
    val Zap = glyph("M4 14a1 1 0 0 1-.78-1.63l9.9-10.2a.5.5 0 0 1 .86.46l-1.92 6.02A1 1 0 0 0 13 10h7a1 1 0 0 1 .78 1.63l-9.9 10.2a.5.5 0 0 1-.86-.46l1.92-6.02A1 1 0 0 0 11 14z")
    val Cursor = glyph("M4.04 4.66a.5.5 0 0 1 .62-.62l15.9 5.98a.5.5 0 0 1-.05.95l-6.1 1.5a2 2 0 0 0-1.45 1.45l-1.5 6.1a.5.5 0 0 1-.95.05z")
    val CreditCard = glyph(rect(2f, 5f, 20f, 14f, 2f), "M2 10h20")
    val Cpu = glyph(rect(4f, 4f, 16f, 16f, 2f), rect(9f, 9f, 6f, 6f, 1f), "M15 2v2", "M15 20v2", "M2 15h2", "M2 9h2", "M20 15h2", "M20 9h2", "M9 2v2", "M9 20v2")
    val Building = glyph(rect(4f, 2f, 16f, 20f, 2f), "M9 22v-4h6v4", "M8 6h.01", "M16 6h.01", "M12 6h.01", "M12 10h.01", "M12 14h.01", "M16 10h.01", "M16 14h.01", "M8 10h.01", "M8 14h.01")
    val Plug = glyph("M12 22v-5", "M9 8V2", "M15 8V2", "M18 8v5a4 4 0 0 1-4 4h-4a4 4 0 0 1-4-4V8Z")
    val Sliders = glyph("M20 7h-9", "M14 17H5", circle(17f, 17f, 3f), circle(7f, 7f, 3f))
    val ExternalLink = glyph("M15 3h6v6", "M10 14 21 3", "M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6")
    val Calendar = glyph(rect(3f, 4f, 18f, 18f, 2f), "M16 2v4", "M8 2v4", "M3 10h18")
    val Inbox = NomiIcons.Inbox
    val Heart = glyph("M19 14c1.49-1.46 3-3.21 3-5.5A5.5 5.5 0 0 0 16.5 3c-1.76 0-3 .5-4.5 2-1.5-1.5-2.74-2-4.5-2A5.5 5.5 0 0 0 2 8.5c0 2.3 1.5 4.05 3 5.5l7 7Z")
    val Signal = glyph("M2 20h.01", "M7 20v-4", "M12 20v-8", "M17 20V8")
    val Battery = glyph(rect(2f, 7f, 16f, 10f, 2f), "M22 11v2")
}
