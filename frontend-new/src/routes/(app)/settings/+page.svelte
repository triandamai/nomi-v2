<script lang="ts">
	import { goto } from '$app/navigation';
	import { Brain, Building2, Cpu, LogOut, Plug, Settings2, User, Users } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import { phoneView } from '$lib/phone.svelte';
	import Icon from '$lib/ds/foundations/Icon.svelte';
	import LargeTitle from '$lib/ds/chrome/LargeTitle.svelte';
	import ThemePickerButton from '$lib/ds/chrome/ThemePickerButton.svelte';
	import PhonePerson from '$lib/ds/settings/PhonePerson.svelte';
	import PhonePlanCard from '$lib/ds/settings/PhonePlanCard.svelte';
	import PhoneSettingsGroup, { type SettingsRow } from '$lib/ds/settings/PhoneSettingsGroup.svelte';

	// The You tab (Flow / Mobile — Home, Inbox & You, step 3 O1SXw): the large title, who's signed in
	// (opening Profile), the plan card (opening Plan & usage), the settings pages in the groups the
	// desktop's settings nav uses, and Log out. A window wider than a phone goes on to Profile, as the
	// server sends a desktop there.
	let { data } = $props();
	const guess = () => data.phone;
	const phone = phoneView(guess());
	$effect(() => {
		if (!phone.current) void goto('/settings/profile', { replaceState: true });
	});

	const groups = $derived<{ label: string; rows: SettingsRow[] }[]>([
		{
			label: m.settings_account(),
			rows: [
				{ href: '/settings/profile', icon: User, label: m.settings_profile() },
				{ href: '/settings/general', icon: Settings2, label: m.settings_general() },
			],
		},
		{
			label: m.settings_ai(),
			rows: [
				{ href: '/settings/crew', icon: Users, label: m.settings_crew(), meta: data.crew.rows.length ? String(data.crew.rows.length) : undefined },
				{ href: '/settings/models', icon: Cpu, label: m.settings_models() },
				{ href: '/settings/memory', icon: Brain, label: m.settings_memory() },
			],
		},
		{
			label: m.settings_workspace(),
			rows: [
				{ href: '/settings/workspaces', icon: Building2, label: m.settings_workspaces(), meta: data.workspaces.length ? String(data.workspaces.length) : undefined },
				{ href: '/settings/connections', icon: Plug, label: m.settings_connections() },
			],
		},
	]);
	const detail = $derived(data.person.username ? m.phone_you_detail({ username: data.person.username }) : m.settings_profile());
</script>

<svelte:head><title>{m.nav_you()} · {m.app_name()}</title></svelte:head>

{#if phone.current}
	<div class="flex h-full flex-col overflow-y-auto pb-6">
		<LargeTitle title={m.nav_you()}>
			{#snippet trailing()}<ThemePickerButton />{/snippet}
		</LargeTitle>
		<div class="flex flex-col gap-5 px-4 pt-1">
			<PhonePerson name={data.person.name} {detail} avatar={data.person.avatar ?? undefined} href="/settings/profile" />
			{#if data.usage}
				<PhonePlanCard plan={data.usage.card.plan} used={data.usage.card.used} caption={data.usage.card.caption} href="/settings/plan" link={m.settings_plan()} />
			{/if}
			{#each groups as group (group.label)}
				<PhoneSettingsGroup label={group.label} rows={group.rows} />
			{/each}
			<form method="POST" action="/sign-out">
				<button type="submit" class="flex h-13 w-full cursor-pointer items-center gap-3 rounded-xl bg-surface px-3.5 text-phone text-danger outline -outline-offset-1 outline-border focus-visible:outline-2 focus-visible:outline-focus">
					<Icon icon={LogOut} size={18} />
					{m.settings_logout()}
				</button>
			</form>
		</div>
	</div>
{/if}
