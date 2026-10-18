package ai.nomi.core

/** How long a screen's flows keep running after its last collector left (configuration changes resubscribe within it). */
const val STATE_STOP_TIMEOUT_MS = 5_000L

/** Shown for a chat the server has not named yet. */
const val UNTITLED_CHAT = "New chat"

/** The quiet meta line on an unread chat or notification row. */
const val UNREAD_META = "New"
