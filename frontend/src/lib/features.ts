// Every place in the app, for the navigation drawer, the More page and the drawer settings.
// Home, Chats and Projects are always in the drawer; the rest can be pinned there in any order.

import type { Component } from 'svelte';
import { m } from '$lib/paraglide/messages';
import IconHome from '$lib/components/icons/IconHome.svelte';
import IconChatBubble from '$lib/components/icons/IconChatBubble.svelte';
import IconFolder from '$lib/components/icons/IconFolder.svelte';
import IconWallet from '$lib/components/icons/IconWallet.svelte';
import IconBell from '$lib/components/icons/IconBell.svelte';
import IconMemory from '$lib/components/icons/IconMemory.svelte';
import IconInbox from '$lib/components/icons/IconInbox.svelte';
import IconAgents from '$lib/components/icons/IconAgents.svelte';
import IconLink from '$lib/components/icons/IconLink.svelte';
import IconChip from '$lib/components/icons/IconChip.svelte';
import IconCard from '$lib/components/icons/IconCard.svelte';
import IconSettings from '$lib/components/icons/IconSettings.svelte';
import IconPerson from '$lib/components/icons/IconPerson.svelte';

import { type PinnableId } from '$lib/drawerPins';

export { PINNABLE, DEFAULT_PINS, isPinnable, resolvePins, moveItem, type PinnableId } from '$lib/drawerPins';
export type FeatureId = 'home' | 'chats' | 'projects' | PinnableId | 'preferences' | 'profile';

/** Always in the drawer, in this order, above the pins. */
export const PERMANENT: FeatureId[] = ['home', 'chats', 'projects'];

export interface Feature {
	id: FeatureId;
	href: string;
	label: () => string;
	description: () => string;
	icon: Component<{ size?: number }>;
}

export const FEATURES: Record<FeatureId, Feature> = {
	home: { id: 'home', href: '/', label: m.nav_home, description: m.feature_home, icon: IconHome },
	chats: { id: 'chats', href: '/chats', label: m.nav_chats, description: m.feature_chats, icon: IconChatBubble },
	projects: { id: 'projects', href: '/projects', label: m.nav_projects, description: m.feature_projects, icon: IconFolder },
	money: { id: 'money', href: '/money', label: m.nav_money, description: m.feature_money, icon: IconWallet },
	reminders: { id: 'reminders', href: '/reminders', label: m.nav_reminders, description: m.feature_reminders, icon: IconBell },
	memory: { id: 'memory', href: '/memory', label: m.nav_memory, description: m.feature_memory, icon: IconMemory },
	notifications: { id: 'notifications', href: '/notifications', label: m.notif_title, description: m.feature_notifications, icon: IconInbox },
	crew: { id: 'crew', href: '/crew', label: m.nav_crew, description: m.feature_crew, icon: IconAgents },
	connections: { id: 'connections', href: '/connections', label: m.nav_connections, description: m.feature_connections, icon: IconLink },
	models: { id: 'models', href: '/models', label: m.nav_model, description: m.feature_models, icon: IconChip },
	billing: { id: 'billing', href: '/billing', label: m.nav_billing, description: m.feature_billing, icon: IconCard },
	preferences: { id: 'preferences', href: '/preferences', label: m.nav_preferences, description: m.feature_preferences, icon: IconSettings },
	profile: { id: 'profile', href: '/profile', label: m.nav_profile, description: m.feature_profile, icon: IconPerson }
};

/** The More page's sections, in order. */
export const MORE_SECTIONS: { title: () => string; ids: FeatureId[] }[] = [
	{ title: m.more_section_main, ids: ['home', 'chats', 'projects'] },
	{ title: m.more_section_life, ids: ['money', 'reminders', 'memory', 'notifications'] },
	{ title: m.more_section_crew, ids: ['crew', 'connections', 'models'] },
	{ title: m.more_section_account, ids: ['billing', 'preferences', 'profile'] }
];
