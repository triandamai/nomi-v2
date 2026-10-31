<script lang="ts">
	import { ArrowLeft, Brain, Building2, CreditCard, Cpu, LogOut, Plug, Settings2, Shield, User, Users } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import NavItem from '../navigation/NavItem.svelte';
	import Search from '../inputs/Search.svelte';
	import Avatar from '../identity/Avatar.svelte';
	import { setRowStyle } from '../navigation/rowStyle';

	// Twin of frontend-new's SettingsNav as an M3 navigation drawer: back to Nomi (esc), search,
	// the groups of settings, and the person with Log out and the app's version at the foot.
	const icons = { user: User, 'settings-2': Settings2, 'credit-card': CreditCard, users: Users, cpu: Cpu, brain: Brain, 'building-2': Building2, plug: Plug, shield: Shield };
	type Item = { href: string; icon: keyof typeof icons; label: string; count?: number | string };
	type Props = {
		backHref: string;
		groups: { label: string; items: Item[] }[];
		active?: string;
		user: { name: string; detail?: string; avatar?: string };
		version?: string;
		query?: string;
		onlogout?: () => void;
	};
	let { backHref, groups, active, user, version, query = $bindable(''), onlogout }: Props = $props();

	setRowStyle({ dense: true, icon: 'text-3' });
</script>

<nav aria-label={m.settings_nav()} class="drawer">
	<a href={backHref} class="back">
		<Icon icon={ArrowLeft} size={20} />
		<span class="grow">{m.settings_back()}</span>
		<span class="meta">esc</span>
	</a>
	<Search placeholder={m.settings_search()} shortcut="/" plain bind:value={query} />
	{#each groups as group (group.label)}
		<div class="group">
			<p class="nomi-meta heading">{group.label}</p>
			{#each group.items as item (item.href)}
				<NavItem href={item.href} icon={icons[item.icon]} label={item.label} count={item.count} active={item.href === active} />
			{/each}
		</div>
	{/each}
	<div class="foot">
		<div class="person">
			<Avatar name={user.name} src={user.avatar} />
			<span class="who">
				<span class="name">{user.name}</span>
				{#if user.detail}<span class="meta">{user.detail}</span>{/if}
			</span>
		</div>
		{#if onlogout}
			<button type="button" class="back" onclick={onlogout}>
				<Icon icon={LogOut} size={20} />
				<span class="grow">{m.settings_logout()}</span>
				{#if version}<span class="meta">{version}</span>{/if}
			</button>
		{:else if version}
			<p class="meta version">{version}</p>
		{/if}
	</div>
</nav>

<style>
	.drawer {
		display: flex;
		height: 100%;
		width: 300px;
		flex-shrink: 0;
		flex-direction: column;
		gap: 12px;
		padding: 12px;
		overflow-y: auto;
		background: var(--md-sys-color-surface-container-low);
		color: var(--md-sys-color-on-surface);
	}
	.back {
		display: flex;
		width: 100%;
		min-height: 48px;
		align-items: center;
		gap: 12px;
		padding: 0 16px 0 12px;
		border: 0;
		border-radius: var(--md-sys-shape-corner-full);
		background: none;
		color: var(--md-sys-color-on-surface-variant);
		text-align: left;
		text-decoration: none;
		cursor: pointer;
		font: var(--md-sys-typescale-label-large-weight) var(--md-sys-typescale-label-large-size) / var(--md-sys-typescale-label-large-line-height) var(--md-sys-typescale-label-large-font);
	}
	.back:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
		color: var(--md-sys-color-on-surface);
	}
	.back:focus-visible {
		outline: 3px solid var(--md-sys-color-primary);
		outline-offset: -3px;
	}
	.grow {
		flex: 1;
	}
	.group {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	.heading {
		margin: 0;
		padding: 8px 16px 4px;
	}
	.foot {
		display: flex;
		flex-direction: column;
		gap: 4px;
		margin-top: auto;
		padding-top: 8px;
		border-top: 1px solid var(--md-sys-color-outline-variant);
	}
	.person {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 8px 12px;
	}
	.who {
		display: flex;
		min-width: 0;
		flex: 1;
		flex-direction: column;
		gap: 2px;
	}
	.name {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font: var(--md-sys-typescale-label-large-weight) var(--md-sys-typescale-label-large-size) / var(--md-sys-typescale-label-large-line-height) var(--md-sys-typescale-label-large-font);
	}
	.meta {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		color: var(--md-sys-color-on-surface-variant);
		font-family: var(--md-ref-typeface-mono);
		font-size: var(--md-sys-typescale-label-small-size);
	}
	.version {
		margin: 0;
		padding: 0 16px;
	}
</style>
