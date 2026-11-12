import { getContext, setContext } from 'svelte';

// Where AppShell puts its panel: beside everything, full height (a chat's crew panel, as the
// Layout shells diagram draws it), or under the top bar, which then runs across it (Home's crew
// column, App / Home Vr4nn). Whoever renders the shell sets it once, as a container sets rowStyle
// for its navigation rows; AppShell's props stay the same as the old app's twin of it.

export type PanelPlacement = { readonly underTopbar: boolean };

const KEY = Symbol('app-shell-panel-placement');

export const setPanelPlacement = (placement: PanelPlacement) => setContext(KEY, placement);

export const panelPlacement = (): PanelPlacement => getContext<PanelPlacement | undefined>(KEY) ?? { underTopbar: false };
