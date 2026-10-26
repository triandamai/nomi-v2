import type { AccentColor, Theme } from '$lib/types';
import { m } from '$lib/paraglide/messages';

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
		get description() {
			return m.theme_canopy();
		},
		light: {"primary": "#0b6b4a", "primarycontainer": "#b9f5c8", "tertiary": "#a33a12", "surface": "#f4f7f2", "surfacecontainerlowest": "#ffffff", "onsurface": "#0f1a14"},
		dark: {"primary": "#7ce0a3", "primarycontainer": "#0b5236", "tertiary": "#ffb596", "surface": "#0d1511", "surfacecontainerlowest": "#08100c", "onsurface": "#e3ede5"},
	},
	{
		value: 'coral-reef',
		label: 'Coral Reef',
		get description() {
			return m.theme_coral();
		},
		light: {"primary": "#a43b32", "primarycontainer": "#ffdad5", "tertiary": "#006a66", "surface": "#fff8f7", "surfacecontainerlowest": "#ffffff", "onsurface": "#221a19"},
		dark: {"primary": "#ffb4aa", "primarycontainer": "#84241e", "tertiary": "#80d5cf", "surface": "#191211", "surfacecontainerlowest": "#130c0c", "onsurface": "#efdfdd"},
	},
	{
		value: 'borneo-dusk',
		label: 'Borneo Dusk',
		get description() {
			return m.theme_borneo();
		},
		light: {"primary": "#924c00", "primarycontainer": "#ffdcc4", "tertiary": "#3d6846", "surface": "#fff8f5", "surfacecontainerlowest": "#ffffff", "onsurface": "#221a14"},
		dark: {"primary": "#ffb780", "primarycontainer": "#6f3800", "tertiary": "#a3d2a8", "surface": "#19120d", "surfacecontainerlowest": "#140d08", "onsurface": "#f0dfd6"},
	},
	{
		value: 'phantom',
		label: 'Phantom',
		get description() {
			return m.theme_phantom();
		},
		light: {"primary": "#625886", "primarycontainer": "#e7deff", "tertiary": "#50606f", "surface": "#fdf8fb", "surfacecontainerlowest": "#ffffff", "onsurface": "#1c1b1e"},
		dark: {"primary": "#ccbff4", "primarycontainer": "#4a406c", "tertiary": "#b8c8d9", "surface": "#141315", "surfacecontainerlowest": "#0f0e10", "onsurface": "#e6e1e4"},
	},
	{
		value: 'senja-jakarta',
		label: 'Senja Jakarta',
		get description() {
			return m.theme_senja();
		},
		light: {"primary": "#a2346a", "primarycontainer": "#ffd9e5", "tertiary": "#914d00", "surface": "#fff8f8", "surfacecontainerlowest": "#ffffff", "onsurface": "#21191c"},
		dark: {"primary": "#ffb0ce", "primarycontainer": "#841a51", "tertiary": "#ffb77d", "surface": "#191114", "surfacecontainerlowest": "#130c0f", "onsurface": "#eedfe2"},
	},
	{
		value: 'lautan',
		label: 'Lautan',
		get description() {
			return m.theme_lautan();
		},
		light: {"primary": "#005db7", "primarycontainer": "#d6e3ff", "tertiary": "#006b5f", "surface": "#f9f9ff", "surfacecontainerlowest": "#ffffff", "onsurface": "#171c25"},
		dark: {"primary": "#a9c7ff", "primarycontainer": "#00468c", "tertiary": "#55dbc7", "surface": "#0f131c", "surfacecontainerlowest": "#0a0e17", "onsurface": "#dfe2ef"},
	},
	{
		value: 'matcha',
		label: 'Matcha',
		get description() {
			return m.theme_matcha();
		},
		light: {"primary": "#4a6800", "primarycontainer": "#b4f700", "tertiary": "#934b00", "surface": "#f8fbe7", "surfacecontainerlowest": "#ffffff", "onsurface": "#191d11"},
		dark: {"primary": "#9ed900", "primarycontainer": "#374e00", "tertiary": "#ffb781", "surface": "#111509", "surfacecontainerlowest": "#0c1005", "onsurface": "#e1e4d1"},
	},
	{
		value: 'saffron',
		label: 'Saffron',
		get description() {
			return m.theme_saffron();
		},
		light: {"primary": "#7c5800", "primarycontainer": "#ffdea8", "tertiary": "#704da0", "surface": "#fff8f3", "surfacecontainerlowest": "#ffffff", "onsurface": "#221a0d"},
		dark: {"primary": "#ffba22", "primarycontainer": "#5e4200", "tertiary": "#d8b9ff", "surface": "#191206", "surfacecontainerlowest": "#140d03", "onsurface": "#f0e0ca"},
	},
	{
		value: 'lagoon',
		label: 'Lagoon',
		get description() {
			return m.theme_lagoon();
		},
		light: {"primary": "#006874", "primarycontainer": "#97f0ff", "tertiary": "#9d4223", "surface": "#eefcfe", "surfacecontainerlowest": "#ffffff", "onsurface": "#111d20"},
		dark: {"primary": "#00daf1", "primarycontainer": "#004f58", "tertiary": "#ffb59d", "surface": "#091517", "surfacecontainerlowest": "#051012", "onsurface": "#d7e5e7"},
	},
	{
		value: 'orchid',
		label: 'Orchid',
		get description() {
			return m.theme_orchid();
		},
		light: {"primary": "#9900cf", "primarycontainer": "#f8d8ff", "tertiary": "#9b432c", "surface": "#fff7fb", "surfacecontainerlowest": "#ffffff", "onsurface": "#201922"},
		dark: {"primary": "#ecb2ff", "primarycontainer": "#74009f", "tertiary": "#ffb4a1", "surface": "#18111a", "surfacecontainerlowest": "#130c15", "onsurface": "#ecdeec"},
	},
	{
		value: 'ruby',
		label: 'Ruby',
		get description() {
			return m.theme_ruby();
		},
		light: {"primary": "#c00016", "primarycontainer": "#ffdad6", "tertiary": "#865300", "surface": "#fff8f7", "surfacecontainerlowest": "#ffffff", "onsurface": "#271816"},
		dark: {"primary": "#ffb4ac", "primarycontainer": "#93000e", "tertiary": "#ffb963", "surface": "#1e100e", "surfacecontainerlowest": "#180b09", "onsurface": "#f9dcd9"},
	},
	{
		value: 'graphite',
		label: 'Graphite',
		get description() {
			return m.theme_graphite();
		},
		light: {"primary": "#55606a", "primarycontainer": "#d8e4f0", "tertiary": "#725b2e", "surface": "#fbf9fa", "surfacecontainerlowest": "#ffffff", "onsurface": "#1b1c1d"},
		dark: {"primary": "#bcc8d3", "primarycontainer": "#3d4852", "tertiary": "#e1c28c", "surface": "#131314", "surfacecontainerlowest": "#0d0e0f", "onsurface": "#e4e2e3"},
	},
];

export const MODES: { value: Theme; label: string }[] = [
	{
		value: 'light',
		get label() {
			return m.mode_light();
		},
	},
	{
		value: 'dark',
		get label() {
			return m.mode_dark();
		},
	},
	{
		value: 'system',
		get label() {
			return m.mode_system();
		},
	},
];

export const DEFAULT_THEME: AccentColor = 'lagoon';

export function isThemeName(value: unknown): value is AccentColor {
	return typeof value === 'string' && THEMES.some((t) => t.value === value);
}
