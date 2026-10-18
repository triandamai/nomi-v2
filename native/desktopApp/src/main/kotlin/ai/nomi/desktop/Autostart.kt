package ai.nomi.desktop

import java.nio.file.Files
import java.nio.file.Path

private const val APP_ID = "ai.nomi.app"
private const val WINDOWS_RUN_KEY = "Software\\Microsoft\\Windows\\CurrentVersion\\Run"
private const val WINDOWS_VALUE_NAME = "Nomi"

interface RegistryAdapter {
    fun setStringValue(keyPath: String, name: String, value: String)
    fun deleteValue(keyPath: String, name: String)
    fun getStringValue(keyPath: String, name: String): String?
}

/**
 * Configures start-on-login across macOS (`~/Library/LaunchAgents/ai.nomi.app.plist`), Windows (`HKCU\...\Run`),
 * and Linux (`~/.config/autostart/ai.nomi.app.desktop`).
 */
class AutostartManager(
    private val os: DesktopOs,
    private val homeDir: Path,
    private val executableCommand: String,
    private val registry: RegistryAdapter? = null,
) {
    val macLaunchAgentPath: Path
        get() = homeDir.resolve("Library/LaunchAgents/$APP_ID.plist")

    val linuxDesktopEntryPath: Path
        get() = homeDir.resolve(".config/autostart/$APP_ID.desktop")

    fun enable() {
        when (os) {
            DesktopOs.MacOS -> {
                Files.createDirectories(macLaunchAgentPath.parent)
                Files.writeString(macLaunchAgentPath, buildMacLaunchAgentPlist(executableCommand))
            }
            DesktopOs.Windows -> {
                registry?.setStringValue(WINDOWS_RUN_KEY, WINDOWS_VALUE_NAME, "\"$executableCommand\" --background")
            }
            DesktopOs.LinuxX11, DesktopOs.LinuxWayland -> {
                Files.createDirectories(linuxDesktopEntryPath.parent)
                Files.writeString(linuxDesktopEntryPath, buildLinuxDesktopEntry(executableCommand))
            }
            DesktopOs.Other -> Unit
        }
    }

    fun disable() {
        when (os) {
            DesktopOs.MacOS -> Files.deleteIfExists(macLaunchAgentPath)
            DesktopOs.Windows -> registry?.deleteValue(WINDOWS_RUN_KEY, WINDOWS_VALUE_NAME)
            DesktopOs.LinuxX11, DesktopOs.LinuxWayland -> Files.deleteIfExists(linuxDesktopEntryPath)
            DesktopOs.Other -> Unit
        }
    }

    fun isEnabled(): Boolean = when (os) {
        DesktopOs.MacOS -> Files.exists(macLaunchAgentPath)
        DesktopOs.Windows -> registry?.getStringValue(WINDOWS_RUN_KEY, WINDOWS_VALUE_NAME) != null
        DesktopOs.LinuxX11, DesktopOs.LinuxWayland -> Files.exists(linuxDesktopEntryPath)
        DesktopOs.Other -> false
    }
}

internal fun buildMacLaunchAgentPlist(executable: String): String = """
    <?xml version="1.0" encoding="UTF-8"?>
    <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
    <plist version="1.0">
    <dict>
        <key>Label</key>
        <string>$APP_ID</string>
        <key>ProgramArguments</key>
        <array>
            <string>$executable</string>
            <string>--background</string>
        </array>
        <key>RunAtLoad</key>
        <true/>
    </dict>
    </plist>
""".trimIndent()

internal fun buildLinuxDesktopEntry(executable: String): String = """
    [Desktop Entry]
    Type=Application
    Name=Nomi
    Exec=$executable --background
    Terminal=false
    X-GNOME-Autostart-enabled=true
""".trimIndent()
