<script lang="ts">
	import { ArrowLeft } from '@lucide/svelte';
	import { invalidate } from '$app/navigation';
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { pageAction } from '$lib/pageAction';
	import { crewOf } from '$lib/adapters/shell';
	import { dayMonth } from '$lib/settings/profile';
	import { chatChoices, codingChoices, codingValue } from '$lib/settings/models';
	import { chosenMember, crewGroups, groupOf, modelRole, type CrewGroupKey } from '$lib/settings/crew';
	import type { CrewMember } from '$lib/adapters/api';
	import { toasts } from '$lib/ds/overlays/toasts.svelte';
	import Icon from '$lib/ds/foundations/Icon.svelte';
	import Search from '$lib/ds/inputs/Search.svelte';
	import Select from '$lib/ds/inputs/Select.svelte';
	import Tabs from '$lib/ds/inputs/Tabs.svelte';
	import CrewGroup from '$lib/ds/settings/CrewGroup.svelte';
	import CrewMemberRow from '$lib/ds/settings/CrewMemberRow.svelte';
	import CrewMemberHeader from '$lib/ds/settings/CrewMemberHeader.svelte';
	import CrewDetailRow from '$lib/ds/settings/CrewDetailRow.svelte';
	import PersonalityVersions from '$lib/ds/settings/PersonalityVersions.svelte';

	// Settings › Crew (App / Settings / Crew vVUeh): the roster in the pen's groups with a search, and
	// the chosen member: their header, how they work (their model where it's the person's to pick:
	// Nomi's chat model and Koda's coding one, through Models & providers' actions; what they're
	// doing now; when they step in) and, for Personality, the versions of how Nomi speaks, any of
	// which can be brought back. On a phone the roster comes first and a member opens on its own.
	// Left out (AGENTS.md §9, BACKEND_TODO.md): adding members and Templates, switching one off,
	// permissions, soul.md and each member's activity.
	let { data } = $props();

	const PATH = '/settings/crew';
	const language = getLocale();
	const GROUPS: Record<CrewGroupKey, { label: () => string; tag: () => string }> = {
		core: { label: m.crew_group_core, tag: m.crew_tag_core },
		specialists: { label: m.crew_group_specialists, tag: m.crew_tag_specialist },
		custom: { label: m.crew_group_custom, tag: m.crew_tag_custom },
	};

	let query = $state('');
	const groups = $derived(crewGroups(data.members, query));
	const asked = $derived(page.url.searchParams.get('member'));
	const member = $derived(chosenMember(data.members, asked));
	const role = $derived(member ? modelRole(member.agent_type) : null);

	const stateOf = (one: CrewMember) =>
		one.state === 'working' ? m.crew_state_working() : one.state === 'waiting' ? m.crew_state_waiting() : one.state === 'done' ? m.crew_state_done() : m.crew_state_on();
	const href = (one: CrewMember) => `?member=${encodeURIComponent(one.agent_type)}`;
	const stepsIn = (one: CrewMember) =>
		one.agent_type === 'chitchat' ? m.crew_steps_nomi() : one.agent_type === 'supervisor' ? m.crew_steps_supervisor() : m.crew_steps_in({ name: one.name });

	let tab = $state('overview');
	$effect(() => {
		if (member?.agent_type !== 'personality') tab = 'overview';
	});

	async function choose(action: 'setDefault' | 'setCoding', fields: Record<string, string>) {
		const result = await pageAction('/settings/models', action, fields);
		if (!result.ok) {
			if (result.message) toasts.show({ kind: 'error', title: m.general_save_failed(), detail: result.message });
		} else toasts.show({ kind: 'success', title: m.crew_model_saved() });
		await invalidate('app:crew');
	}

	let restoring = $state<number>();
	async function restore(version: number) {
		restoring = version;
		const result = await pageAction(PATH, 'restore', { version: String(version) });
		restoring = undefined;
		if (!result.ok) {
			if (result.message) toasts.show({ kind: 'error', title: m.action_failed(), detail: result.message });
			return;
		}
		toasts.show({ kind: 'success', title: m.crew_version_restored({ version }) });
		await invalidate('app:crew');
	}
</script>

<svelte:head><title>{m.settings_crew()} · {m.app_name()}</title></svelte:head>

<div class="flex h-full min-h-0">
	<nav aria-label={m.crew_roster()} class={['min-h-0 w-full shrink-0 flex-col gap-3 overflow-y-auto px-3 py-4 lg:flex lg:w-90 lg:border-r lg:border-border lg:pr-2.75', asked ? 'hidden' : 'flex']}>
		<Search placeholder={m.crew_search()} bind:value={query} />
		{#each groups as group (group.key)}
			<CrewGroup label={GROUPS[group.key].label()} note={group.key === 'core' ? m.crew_always_on() : String(group.members.length)}>
				{#each group.members as one (one.agent_type)}
					<li>
						<CrewMemberRow
							agent={one.is_dynamic ? undefined : crewOf(one.agent_type)}
							name={one.name}
							role={one.role}
							state={stateOf(one)}
							tone={one.state === 'waiting' ? 'warning' : 'quiet'}
							selected={one.agent_type === member?.agent_type}
							href={href(one)}
						/>
					</li>
				{/each}
			</CrewGroup>
		{:else}
			<p class="px-2 text-caption text-text-3">{m.crew_none_match()}</p>
		{/each}
	</nav>

	{#if member}
		<section aria-label={member.name} class={['min-h-0 min-w-0 flex-1 flex-col overflow-y-auto', asked ? 'flex' : 'hidden lg:flex']}>
			<a href={PATH} data-sveltekit-noscroll class="mx-4 mt-3 inline-flex items-center gap-1.5 self-start rounded-sm text-ui text-text-2 outline-offset-2 hover:text-text lg:hidden">
				<Icon icon={ArrowLeft} />{m.crew_back()}
			</a>
			<div class="p-4 lg:p-0">
				<CrewMemberHeader
					agent={member.is_dynamic ? undefined : crewOf(member.agent_type)}
					name={member.name}
					tag={GROUPS[groupOf(member)].tag()}
					detail={member.role}
				/>
			</div>
			<div class="px-4 lg:px-7">
				<Tabs
					label={m.crew_tabs({ name: member.name })}
					tabs={member.agent_type === 'personality' && data.personality
						? [
								{ value: 'overview', label: m.crew_tab_overview() },
								{ value: 'versions', label: m.crew_tab_versions({ count: data.personality.length }) },
							]
						: [{ value: 'overview', label: m.crew_tab_overview() }]}
					bind:value={tab}
				>
					{#snippet panel(value)}
						<div class="flex flex-col gap-4 pt-4 pb-10">
							{#if value === 'versions' && data.personality}
								{@const current = data.personality.find((one) => one.is_current)}
								<PersonalityVersions
									title="personality"
									count={current ? m.crew_versions_count({ current: `v${current.version}`, count: data.personality.length }) : m.crew_versions_none()}
									empty={m.crew_versions_empty()}
									versions={data.personality.map((one) => ({ version: one.version, description: one.description, date: dayMonth(new Date(one.created_at), language), current: one.is_current }))}
									labels={{ current: m.crew_version_current(), restore: m.crew_version_restore() }}
									busy={restoring}
									onrestore={restore}
								/>
							{:else}
								<section aria-labelledby="works-title" class="flex flex-col gap-2.5">
									<h3 id="works-title" class="text-ui font-medium text-text">{m.crew_how_works({ name: member.name })}</h3>
									<div class="flex flex-col border-t border-border">
										{#if role === 'chat' && data.options}
											<CrewDetailRow title={m.crew_model()} description={m.crew_model_nomi()}>
												{#snippet control()}
													<div class="w-64 max-w-full">
														<Select label={m.crew_model()} options={chatChoices(data.options!.groups)} value={data.options!.current?.key ?? ''} mono onchange={(key) => choose('setDefault', { key })} />
													</div>
												{/snippet}
											</CrewDetailRow>
										{:else if role === 'coding' && data.models}
											<CrewDetailRow title={m.crew_model()} description={m.models_coding_body()}>
												{#snippet control()}
													<div class="w-64 max-w-full">
														<Select label={m.crew_model()} options={codingChoices(data.models!)} value={codingValue(data.models!.coding)} mono onchange={(value) => choose('setCoding', { value })} />
													</div>
												{/snippet}
											</CrewDetailRow>
										{:else}
											<CrewDetailRow title={m.crew_model()} description={m.crew_model_follows()} />
										{/if}
										<CrewDetailRow title={m.crew_steps()} description={stepsIn(member)} />
										<CrewDetailRow title={m.crew_now()} description={member.status || stateOf(member)} />
									</div>
								</section>
							{/if}
						</div>
					{/snippet}
				</Tabs>
			</div>
		</section>
	{/if}
</div>
