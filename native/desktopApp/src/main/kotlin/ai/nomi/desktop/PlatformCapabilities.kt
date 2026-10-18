package ai.nomi.desktop

enum class DesktopOs {
    MacOS,
    Windows,
    LinuxX11,
    LinuxWayland,
    Other,
    ;

    /** Wayland forbids client-positioned top-most windows and global cursor coordinates, so the island is disabled there. */
    val supportsIsland: Boolean
        get() = this == MacOS || this == Windows || this == LinuxX11
}

/**
 * Resolves the desktop OS and display server from [osName] (`System.getProperty("os.name")`) and environment variables.
 */
fun detectDesktopOs(
    osName: String = System.getProperty("os.name").orEmpty(),
    env: Map<String, String> = System.getenv(),
): DesktopOs {
    val lower = osName.lowercase()
    return when {
        lower.contains("mac") || lower.contains("darwin") -> DesktopOs.MacOS
        lower.contains("win") -> DesktopOs.Windows
        lower.contains("nux") || lower.contains("nix") -> {
            val sessionType = env["XDG_SESSION_TYPE"].orEmpty().lowercase()
            val waylandDisplay = env["WAYLAND_DISPLAY"].orEmpty()
            if (sessionType == "wayland" || waylandDisplay.isNotBlank()) {
                DesktopOs.LinuxWayland
            } else {
                DesktopOs.LinuxX11
            }
        }
        else -> DesktopOs.Other
    }
}
