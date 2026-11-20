import { getContext, setContext } from 'svelte';
import type { Theme } from '$lib/theme';

export type ThemeContext = {
	readonly theme: Theme;
	changeTheme: (theme: Theme) => Promise<void> | void;
};

const KEY = Symbol('nomi-theme-context');

export const setThemeContext = (ctx: ThemeContext) => setContext(KEY, ctx);
export const getThemeContext = (): ThemeContext | undefined => getContext<ThemeContext | undefined>(KEY);
