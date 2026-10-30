<script lang="ts">
	import { ArrowLeft, Brain, Building2, CreditCard, Cpu, LogOut, Plug, Settings2, Shield, User, Users } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import NavItem from '../navigation/NavItem.svelte';
	import Search from '../inputs/Search.svelte';
	import Avatar from '../identity/Avatar.svelte';
	import { setRowStyle } from '../navigation/rowStyle';

	// Agents & app chrome → Settings nav: back to Nomi (esc), search, the groups of settings, and the
	// person with Log out and the app's version at the foot.
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

<nav aria-label={m.settings_nav()} class="flex h-full w-62 shrink-0 flex-col gap-3.5 border-r border-border bg-bg py-4 pr-2.75 pl-3">
	<a href={backHref} class="flex h-8 items-center gap-2.5 rounded-sm px-2.5 text-ui text-text-2 outline-offset-1 hover:bg-surface-2 hover:text-text pointer-coarse:min-h-11">
		<span class="inline-flex text-text-3"><Icon icon={ArrowLeft} /></span>
		<span class="flex-1">{m.settings_back()}</span>
		<span class="font-mono text-label text-text-3">esc</span>
	</a>
	<Search placeholder={m.settings_search()} shortcut="/" plain bind:value={query} />
	{#each groups as group (group.label)}
		<div class="flex flex-col gap-0.5">
			<p class="px-2.5 pb-1 font-mono text-label tracking-label text-text-3 uppercase">{group.label}</p>
			{#each group.items as item (item.href)}
				<NavItem href={item.href} icon={icons[item.icon]} label={item.label} count={item.count} active={item.href === active} />
			{/each}
		</div>
	{/each}
	<div class="mt-auto flex flex-col gap-1 border-t border-border pt-1.75">
		<div class="flex items-center gap-2.5 px-2 py-1.5">
			<Avatar name={user.name} src={user.avatar} />
			<span class="flex min-w-0 flex-1 flex-col gap-0.5">
				<span class="truncate text-ui text-text">{user.name}</span>
				{#if user.detail}<span class="truncate font-mono text-label text-text-3">{user.detail}</span>{/if}
			</span>
		</div>
		{#if onlogout}
			<button type="button" class="flex h-7.5 w-full cursor-pointer items-center gap-2.5 rounded-sm px-2.5 text-left text-ui text-text-2 outline-offset-1 hover:bg-surface-2 hover:text-text pointer-coarse:min-h-11" onclick={onlogout}>
				<span class="inline-flex text-text-3"><Icon icon={LogOut} size={14} /></span>
				<span class="flex-1">{m.settings_logout()}</span>
				{#if version}<span class="font-mono text-label text-text-3">{version}</span>{/if}
			</button>
		{:else if version}
			<p class="px-2.5 font-mono text-label text-text-3">{version}</p>
		{/if}
	</div>
</nav>
