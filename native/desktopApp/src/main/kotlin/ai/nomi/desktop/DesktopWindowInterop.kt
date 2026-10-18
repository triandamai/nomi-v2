package ai.nomi.desktop

import androidx.compose.ui.awt.ComposeWindow
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.platform.win32.User32
import com.sun.jna.platform.win32.WinDef
import com.sun.jna.platform.win32.WinUser

private const val NS_STATUS_WINDOW_LEVEL = 26L
private const val NS_WINDOW_COLLECTION_CAN_JOIN_ALL_SPACES = 1L shl 0
private const val NS_WINDOW_COLLECTION_STATIONARY = 1L shl 4
private const val NS_WINDOW_COLLECTION_IGNORES_CYCLE = 1L shl 6
private const val NS_WINDOW_COLLECTION_FULL_SCREEN_AUXILIARY = 1L shl 8

private const val WS_EX_TRANSPARENT = 0x00000020
private const val WS_EX_TOOLWINDOW = 0x00000080
private const val WS_EX_TOPMOST = 0x00000008
private const val WS_EX_LAYERED = 0x00080000
private const val WS_EX_NOACTIVATE = 0x08000000

private interface ObjCRuntime : Library {
    fun objc_getClass(name: String): Pointer?
    fun sel_registerName(name: String): Pointer?
    fun objc_msgSend(receiver: Pointer, selector: Pointer): Pointer?
    fun objc_msgSend(receiver: Pointer, selector: Pointer, arg: Long): Pointer?
    fun objc_msgSend(receiver: Pointer, selector: Pointer, arg: Boolean): Pointer?
}

/**
 * Applies OS-level window flags to the Dynamic Island window (above menu bar / joins all Spaces on macOS,
 * tool-window / no-activate on Windows) and toggles mouse click-through when the cursor is outside the pill.
 */
object DesktopWindowInterop {
    private val objc: ObjCRuntime? by lazy {
        runCatching { Native.load("objc", ObjCRuntime::class.java) }.getOrNull()
    }

    fun configureIslandWindow(window: ComposeWindow, os: DesktopOs) {
        runCatching {
            when (os) {
                DesktopOs.MacOS -> configureMacIslandWindow(window)
                DesktopOs.Windows -> configureWindowsIslandWindow(window)
                else -> Unit
            }
        }
    }

    fun setIslandClickThrough(window: ComposeWindow, os: DesktopOs, clickThrough: Boolean) {
        runCatching {
            when (os) {
                DesktopOs.MacOS -> setMacClickThrough(window, clickThrough)
                DesktopOs.Windows -> setWindowsClickThrough(window, clickThrough)
                else -> Unit
            }
        }
    }

    private fun resolveNsWindow(window: ComposeWindow): Pointer? {
        val handle = window.windowHandle
        if (handle == 0L) return null
        val ptr = Pointer(handle)
        val runtime = objc ?: return ptr
        val windowSel = runtime.sel_registerName("window") ?: return ptr
        // ComposeWindow.windowHandle on macOS is an NSWindow* (or NSView* on older builds; respondsToSelector guards it).
        val respondsSel = runtime.sel_registerName("respondsToSelector:")
        if (respondsSel != null) {
            val setLevelSel = runtime.sel_registerName("setLevel:")
            if (setLevelSel != null && runtime.objc_msgSend(ptr, respondsSel, Pointer.nativeValue(setLevelSel)) != null) {
                return ptr
            }
        }
        return runtime.objc_msgSend(ptr, windowSel) ?: ptr
    }

    private fun configureMacIslandWindow(window: ComposeWindow) {
        val runtime = objc ?: return
        val nsWindow = resolveNsWindow(window) ?: return
        runtime.sel_registerName("setLevel:")?.let { sel ->
            runtime.objc_msgSend(nsWindow, sel, NS_STATUS_WINDOW_LEVEL)
        }
        val behavior = NS_WINDOW_COLLECTION_CAN_JOIN_ALL_SPACES or
            NS_WINDOW_COLLECTION_STATIONARY or
            NS_WINDOW_COLLECTION_IGNORES_CYCLE or
            NS_WINDOW_COLLECTION_FULL_SCREEN_AUXILIARY
        runtime.sel_registerName("setCollectionBehavior:")?.let { sel ->
            runtime.objc_msgSend(nsWindow, sel, behavior)
        }
        runtime.sel_registerName("setHasShadow:")?.let { sel ->
            runtime.objc_msgSend(nsWindow, sel, false)
        }
    }

    private fun setMacClickThrough(window: ComposeWindow, clickThrough: Boolean) {
        val runtime = objc ?: return
        val nsWindow = resolveNsWindow(window) ?: return
        runtime.sel_registerName("setIgnoresMouseEvents:")?.let { sel ->
            runtime.objc_msgSend(nsWindow, sel, clickThrough)
        }
    }

    private fun configureWindowsIslandWindow(window: ComposeWindow) {
        val handle = window.windowHandle
        if (handle == 0L) return
        val hwnd = WinDef.HWND(Pointer(handle))
        val current = User32.INSTANCE.GetWindowLong(hwnd, WinUser.GWL_EXSTYLE)
        val updated = current or WS_EX_TOOLWINDOW or WS_EX_NOACTIVATE or WS_EX_TOPMOST or WS_EX_LAYERED
        User32.INSTANCE.SetWindowLong(hwnd, WinUser.GWL_EXSTYLE, updated)
    }

    private fun setWindowsClickThrough(window: ComposeWindow, clickThrough: Boolean) {
        val handle = window.windowHandle
        if (handle == 0L) return
        val hwnd = WinDef.HWND(Pointer(handle))
        val current = User32.INSTANCE.GetWindowLong(hwnd, WinUser.GWL_EXSTYLE)
        val updated = if (clickThrough) {
            current or WS_EX_TRANSPARENT
        } else {
            current and WS_EX_TRANSPARENT.inv()
        }
        if (updated != current) {
            User32.INSTANCE.SetWindowLong(hwnd, WinUser.GWL_EXSTYLE, updated)
        }
    }
}
