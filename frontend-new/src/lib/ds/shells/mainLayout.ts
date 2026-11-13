import { getContext, setContext } from 'svelte';

// How AppShell lays out its content: padded and scrolling as a page (32–56 around, the default),
// or edge to edge (`bleed`) for a screen that scrolls a part of itself and pins another, as a chat
// keeps its composer at the bottom while the thread scrolls (Flow / Chat — Ask → think → answer:
// the thread padded [40,48,0,48], the composer [0,48,24,48]). Set by whoever renders the shell, as
// panelPlacement is, so AppShell's props stay the old app's twin's.

export type MainLayout = { readonly bleed: boolean };

const KEY = Symbol('app-shell-main-layout');

export const setMainLayout = (layout: MainLayout) => setContext(KEY, layout);

export const mainLayout = (): MainLayout => getContext<MainLayout | undefined>(KEY) ?? { bleed: false };
