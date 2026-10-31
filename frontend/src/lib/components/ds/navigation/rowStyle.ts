import { getContext, setContext } from 'svelte';

// How navigation rows sit in the container they're in. The pen's standalone Nav Item is 12px in
// with a text-3 icon; inside the Sidebar rows are 10px in with text-2 icons, and inside the
// Settings nav 10px in with text-3 icons. The container sets it once for its rows.

export type RowStyle = { dense: boolean; icon: 'text-2' | 'text-3' };

const KEY = Symbol('nav-row-style');

export const setRowStyle = (style: RowStyle) => setContext(KEY, style);

export const rowStyle = (): RowStyle => getContext<RowStyle | undefined>(KEY) ?? { dense: false, icon: 'text-3' };
