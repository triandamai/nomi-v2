import type { AccentColor, Theme } from '$lib/types';

/** The colours a theme's preview tile is drawn with: the same values as its block in material3.css. */
export interface ThemeSwatch {
	primary: string;
	primarycontainer: string;
	tertiary: string;
	surface: string;
	surfacecontainerlowest: string;
	onsurface: string;
}

/** Appearance themes. Canopy is Nomi's green (the base palette); each recolours the whole UI. */
export const THEMES: { value: AccentColor; label: string; description: string; light: ThemeSwatch; dark: ThemeSwatch }[] = [
	{
		value: 'canopy',
		label: 'Canopy',
		description: 'Rainforest green under a bright sky. Nomi’s own.',
		light: {"primary": "#0b6b4a", "primarycontainer": "#b9f5c8", "tertiary": "#a33a12", "surface": "#f4f7f2", "surfacecontainerlowest": "#ffffff", "onsurface": "#0f1a14"},
		dark: {"primary": "#7ce0a3", "primarycontainer": "#0b5236", "tertiary": "#ffb596", "surface": "#0d1511", "surfacecontainerlowest": "#08100c", "onsurface": "#e3ede5"},
	},
	{
		value: 'coral-reef',
		label: 'Coral Reef',
		description: 'Warm coral over clear reef water.',
		light: {"primary": "#a43b32", "primarycontainer": "#ffdad5", "tertiary": "#006a66", "surface": "#fff8f7", "surfacecontainerlowest": "#ffffff", "onsurface": "#221a19"},
		dark: {"primary": "#ffb4aa", "primarycontainer": "#84241e", "tertiary": "#80d5cf", "surface": "#191211", "surfacecontainerlowest": "#130c0c", "onsurface": "#efdfdd"},
	},
	{
		value: 'borneo-dusk',
		label: 'Borneo Dusk',
		description: 'Amber light falling through the rainforest.',
		light: {"primary": "#924c00", "primarycontainer": "#ffdcc4", "tertiary": "#3d6846", "surface": "#fff8f5", "surfacecontainerlowest": "#ffffff", "onsurface": "#221a14"},
		dark: {"primary": "#ffb780", "primarycontainer": "#6f3800", "tertiary": "#a3d2a8", "surface": "#19120d", "surfacecontainerlowest": "#140d08", "onsurface": "#f0dfd6"},
	},
	{
		value: 'phantom',
		label: 'Phantom',
		description: 'Quiet violet and mist, nearly grey.',
		light: {"primary": "#625886", "primarycontainer": "#e7deff", "tertiary": "#50606f", "surface": "#fdf8fb", "surfacecontainerlowest": "#ffffff", "onsurface": "#1c1b1e"},
		dark: {"primary": "#ccbff4", "primarycontainer": "#4a406c", "tertiary": "#b8c8d9", "surface": "#141315", "surfacecontainerlowest": "#0f0e10", "onsurface": "#e6e1e4"},
	},
	{
		value: 'senja-jakarta',
		label: 'Senja Jakarta',
		description: 'Magenta sky and orange haze over the city at dusk.',
		light: {"primary": "#a2346a", "primarycontainer": "#ffd9e5", "tertiary": "#914d00", "surface": "#fff8f8", "surfacecontainerlowest": "#ffffff", "onsurface": "#21191c"},
		dark: {"primary": "#ffb0ce", "primarycontainer": "#841a51", "tertiary": "#ffb77d", "surface": "#191114", "surfacecontainerlowest": "#130c0f", "onsurface": "#eedfe2"},
	},
];

export const MODES: { value: Theme; label: string }[] = [
	{ value: 'light', label: 'Light' },
	{ value: 'dark', label: 'Dark' },
	{ value: 'system', label: 'System' },
];

export const DEFAULT_THEME: AccentColor = 'canopy';

export function isThemeName(value: unknown): value is AccentColor {
	return typeof value === 'string' && THEMES.some((t) => t.value === value);
}
