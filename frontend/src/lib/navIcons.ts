// The design-system drawer draws its rows with Lucide icons (NavItem); the app's features carry
// M3 ones. Which Lucide icon each feature is in the drawer.
import type { Component } from 'svelte';
import { Bell, Brain, Building, Cpu, CreditCard, Folder, House, Inbox, LayoutGrid, Link2, MessageSquare, Settings, Sparkles, User, Users, Wallet, type LucideProps } from '@lucide/svelte';
import type { FeatureId } from '$lib/features';

export const NAV_ICONS: Record<FeatureId | 'more', Component<LucideProps>> = {
	home: House,
	chats: MessageSquare,
	projects: Folder,
	showcase: Sparkles,
	money: Wallet,
	reminders: Bell,
	memory: Brain,
	notifications: Inbox,
	crew: Users,
	connections: Link2,
	models: Cpu,
	orgs: Building,
	billing: CreditCard,
	preferences: Settings,
	profile: User,
	more: LayoutGrid,
};
