package ai.nomi.desktop

import androidx.compose.ui.awt.ComposeWindow
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.NativeLibrary
import com.sun.jna.Pointer
import com.sun.jna.platform.win32.User32
import com.sun.jna.platform.win32.WinDef
import com.sun.jna.platform.win32.WinUser
import java.awt.Toolkit
import java.util.concurrent.ConcurrentLinkedQueue

private const val NS_POPUP_MENU_WINDOW_LEVEL = 101L
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
    fun object_getClassName(obj: Pointer): String?
    fun sel_registerName(name: String): Pointer?
    fun objc_msgSend(receiver: Pointer, selector: Pointer): Pointer?
    fun objc_msgSend(receiver: Pointer, selector: Pointer, arg: Long): Pointer?
    fun objc_msgSend(receiver: Pointer, selector: Pointer, arg: Boolean): Pointer?
    fun objc_msgSend(receiver: Pointer, selector: Pointer, x: Double, y: Double): Pointer?
}

private interface DispatchWorkCallback : Callback {
    fun invoke(context: Pointer?)
}

private interface MacSystemLibrary : Library {
    fun pthread_main_np(): Int
    fun dispatch_async_f(queue: Pointer, context: Pointer?, work: DispatchWorkCallback)
}

/**
 * Applies OS-level window flags to the Dynamic Island window (above menu bar / joins all Spaces on macOS,
 * tool-window / no-activate on Windows) and toggles mouse click-through when the cursor is outside the pill.
 */
object DesktopWindowInterop {
    private val objc: ObjCRuntime? by lazy {
        runCatching { Native.load("objc", ObjCRuntime::class.java) }.getOrNull()
    }

    private val macSystem: MacSystemLibrary? by lazy {
        runCatching { Native.load("System", MacSystemLibrary::class.java) }.getOrNull()
    }

    private val macMainQueue: Pointer? by lazy {
        runCatching { NativeLibrary.getInstance("System").getGlobalVariableAddress("_dispatch_main_q") }.getOrNull()
    }

    private val mainThreadTasks = ConcurrentLinkedQueue<() -> Unit>()

    private val mainQueueDrainCallback = object : DispatchWorkCallback {
        override fun invoke(context: Pointer?) {
            while (true) {
                val task = mainThreadTasks.poll() ?: break
                runCatching { task() }
            }
        }
    }

    private fun runOnMacMainThread(block: () -> Unit): Boolean {
        val sys = macSystem ?: return false
        if (sys.pthread_main_np() != 0) {
            runCatching { block() }
            return true
        }
        val queue = macMainQueue ?: return false
        mainThreadTasks.add(block)
        sys.dispatch_async_f(queue, null, mainQueueDrainCallback)
        return true
    }

    fun configureIslandWindow(window: ComposeWindow, os: DesktopOs): Boolean =
        runCatching {
            when (os) {
                DesktopOs.MacOS -> configureMacIslandWindow(window)
                DesktopOs.Windows -> configureWindowsIslandWindow(window)
                else -> true
            }
        }.getOrDefault(false)

    fun positionIslandWindow(window: ComposeWindow, os: DesktopOs, xDp: Int, yDp: Int) {
        runCatching {
            if (os == DesktopOs.MacOS) {
                positionMacIslandWindow(window, xDp, yDp)
            } else {
                window.setLocation(xDp, yDp)
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

    private fun resolveNsWindow(handle: Long): Pointer? {
        if (handle == 0L) return null
        val ptr = Pointer(handle)
        val runtime = objc ?: return ptr
        val className = runCatching { runtime.object_getClassName(ptr) }.getOrNull().orEmpty()
        if (className.contains("Window", ignoreCase = true)) {
            return ptr
        }
        val windowSel = runtime.sel_registerName("window") ?: return ptr
        return runtime.objc_msgSend(ptr, windowSel) ?: ptr
    }

    private fun configureMacIslandWindow(window: ComposeWindow): Boolean {
        val runtime = objc ?: return false
        val handle = window.windowHandle
        if (handle == 0L) return false
        return runOnMacMainThread {
            val nsWindow = resolveNsWindow(handle) ?: return@runOnMacMainThread
            runtime.sel_registerName("setLevel:")?.let { sel ->
                runtime.objc_msgSend(nsWindow, sel, NS_POPUP_MENU_WINDOW_LEVEL)
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
            runtime.sel_registerName("setMovable:")?.let { sel ->
                runtime.objc_msgSend(nsWindow, sel, false)
            }
        }
    }

    private fun positionMacIslandWindow(window: ComposeWindow, xDp: Int, yDp: Int) {
        val runtime = objc ?: return
        val handle = window.windowHandle
        if (handle == 0L) return
        val primaryHeight = runCatching { Toolkit.getDefaultToolkit().screenSize.height.toDouble() }.getOrNull() ?: return
        val cocoaX = xDp.toDouble()
        val cocoaTopY = primaryHeight - yDp.toDouble()
        runOnMacMainThread {
            val nsWindow = resolveNsWindow(handle) ?: return@runOnMacMainThread
            runtime.sel_registerName("setLevel:")?.let { sel ->
                runtime.objc_msgSend(nsWindow, sel, NS_POPUP_MENU_WINDOW_LEVEL)
            }
            runtime.sel_registerName("setFrameTopLeftPoint:")?.let { sel ->
                runtime.objc_msgSend(nsWindow, sel, cocoaX, cocoaTopY)
            }
        }
    }

    private fun setMacClickThrough(window: ComposeWindow, clickThrough: Boolean) {
        val runtime = objc ?: return
        val handle = window.windowHandle
        if (handle == 0L) return
        runOnMacMainThread {
            val nsWindow = resolveNsWindow(handle) ?: return@runOnMacMainThread
            runtime.sel_registerName("setIgnoresMouseEvents:")?.let { sel ->
                runtime.objc_msgSend(nsWindow, sel, clickThrough)
            }
        }
    }

    private fun configureWindowsIslandWindow(window: ComposeWindow): Boolean {
        val handle = window.windowHandle
        if (handle == 0L) return false
        val hwnd = WinDef.HWND(Pointer(handle))
        val current = User32.INSTANCE.GetWindowLong(hwnd, WinUser.GWL_EXSTYLE)
        val updated = current or WS_EX_TOOLWINDOW or WS_EX_NOACTIVATE or WS_EX_TOPMOST or WS_EX_LAYERED
        User32.INSTANCE.SetWindowLong(hwnd, WinUser.GWL_EXSTYLE, updated)
        return true
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
