// The account pages, which share the settings drawer (the design system's SettingsNav) in place
// of the main navigation. Their URLs stay where they are; the drawer goes by path.
import { m } from '$lib/paraglide/messages';

export type SettingsIcon = 'user' | 'settings-2' | 'credit-card' | 'users' | 'cpu' | 'brain' | 'building-2' | 'plug' | 'shield';

export interface SettingsItem {
	href: string;
	icon: SettingsIcon;
	label: string;
}

export interface SettingsGroup {
	label: string;
	items: SettingsItem[];
}

export const SETTINGS_PATHS = ['/profile', '/preferences', '/account', '/billing', '/models', '/providers', '/connections', '/memory'] as const;

/** The settings page `path` is on (its own page or one below it), if any. */
export function activeSetting(path: string): string | undefined {
	return SETTINGS_PATHS.find((href) => path === href || path.startsWith(`${href}/`));
}

export function settingsGroups(): SettingsGroup[] {
	return [
		{
			label: m.more_section_account(),
			items: [
				{ href: '/profile', icon: 'user', label: m.nav_profile() },
				{ href: '/preferences', icon: 'settings-2', label: m.nav_preferences() },
				{ href: '/account', icon: 'shield', label: m.title_account() },
				{ href: '/billing', icon: 'credit-card', label: m.nav_billing() },
			],
		},
		{
			label: m.more_section_crew(),
			items: [
				{ href: '/models', icon: 'cpu', label: m.nav_model() },
				{ href: '/providers', icon: 'building-2', label: m.prov_nav() },
				{ href: '/connections', icon: 'plug', label: m.nav_connections() },
				{ href: '/memory', icon: 'brain', label: m.nav_memory() },
			],
		},
	];
}

/** The settings whose name has `query` in it, any case; a group with none left goes. */
export function filterSettings(groups: SettingsGroup[], query: string): SettingsGroup[] {
	const wanted = query.trim().toLocaleLowerCase();
	if (!wanted) return groups;
	return groups
		.map((group) => ({ ...group, items: group.items.filter((item) => item.label.toLocaleLowerCase().includes(wanted)) }))
		.filter((group) => group.items.length > 0);
}
