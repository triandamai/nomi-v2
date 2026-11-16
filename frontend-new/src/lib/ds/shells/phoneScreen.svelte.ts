import { getContext, setContext } from 'svelte';

// Whether a phone shows a screen inside one of its tabs (a project opened from Projects, as Flow /
// Mobile Projects — Detail panel draws it): such a screen has its own back and no tab bar, as a chat
// does. Whoever renders the shell owns it (phoneChrome reads it); a page sets it while it shows one.

export class PhoneScreen {
	inside = $state(false);
}

const KEY = Symbol('app-shell-phone-screen');

export const setPhoneScreen = (screen: PhoneScreen) => setContext(KEY, screen);

/** The shell's, or one of its own where there's no shell around (a test). */
export const phoneScreen = (): PhoneScreen => getContext<PhoneScreen | undefined>(KEY) ?? new PhoneScreen();
