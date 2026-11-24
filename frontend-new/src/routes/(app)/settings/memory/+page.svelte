<script lang="ts">
	import { Brain, Download, Gauge, Goal, Heart, History, Lightbulb, Repeat, Search as SearchIcon, Sparkles, UserRound, Users } from '@lucide/svelte';
	import { goto, invalidate } from '$app/navigation';
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { pageAction } from '$lib/pageAction';
	import { startChat } from '$lib/chat/start';
	import { dayMonth } from '$lib/settings/profile';
	import { memoryCounts, memoryTopics, strength, topLearner, type MemoryFilter, type StrengthName } from '$lib/settings/memory';
	import type { MemoryItem, MemoryProfileData } from '$lib/adapters/api';
	import { crewOf } from '$lib/adapters/shell';
	import { toasts } from '$lib/ds/overlays/toasts.svelte';
	import Search from '$lib/ds/inputs/Search.svelte';
	import Segmented from '$lib/ds/inputs/Segmented.svelte';
	import SectionLabel from '$lib/ds/status/SectionLabel.svelte';
	import EmptyState from '$lib/ds/lists/EmptyState.svelte';
	import ChatComposer from '$lib/ds/thread/ChatComposer.svelte';
	import MemoryTable from '$lib/ds/settings/MemoryTable.svelte';
	import MemoryTopicRow from '$lib/ds/settings/MemoryTopicRow.svelte';
	import MemoryEntry from '$lib/ds/settings/MemoryEntry.svelte';
	import MemoryProfile from '$lib/ds/settings/MemoryProfile.svelte';
	import MemorySettings, { type MemorySettingsChange, type MemorySettingsValues } from '$lib/ds/settings/MemorySettings.svelte';
	import SettingNote from '$lib/ds/settings/SettingNote.svelte';
	import { phoneView } from '$lib/phone.svelte';
	import { phoneScreen } from '$lib/ds/shells/phoneScreen.svelte';
	import PhoneBackBar from '$lib/ds/chrome/PhoneBackBar.svelte';
	import LargeTitle from '$lib/ds/chrome/LargeTitle.svelte';
	import PhoneSettingsGroup from '$lib/ds/settings/PhoneSettingsGroup.svelte';
	import PhoneFormScreen from '$lib/ds/settings/PhoneFormScreen.svelte';
	import PhoneMemoryProfile from '$lib/ds/settings/PhoneMemoryProfile.svelte';
	import PhoneMemoryCard from '$lib/ds/settings/PhoneMemoryCard.svelte';
	import PhoneButton from '$lib/ds/mobile/PhoneButton.svelte';
	import BottomSheet from '$lib/ds/overlays/BottomSheet.svelte';
	import SheetOption from '$lib/ds/overlays/SheetOption.svelte';
	import ActionSheet from '$lib/ds/overlays/ActionSheet.svelte';
	import TextArea from '$lib/ds/admin/TextArea.svelte';

	// Settings › Memory & context (App / Settings / Memory hANTE): what the crew remembers, by topic
	// (the backend's kinds), each opening to its memories to correct, confirm or forget; the Core
	// and Fading views and a search; and the composer to tell Nomi something to remember or forget,
	// which starts a chat with it. Above the topics, the profile every agent reads (You), edited in
	// place; each topic says who learned most of it, each memory who learned it. Beside them (under
	// them below 1280), Learning and Budget. Left out (AGENTS.md §9): Storage, Context & compression,
	// Import and Forget everything (nothing behind them); the search sits on the page, not the top bar.
	let { data } = $props();

	const PATH = '/settings/memory';
	const language = getLocale();
	const KINDS: Record<string, () => string> = {
		preference: m.memory_kind_preference,
		person: m.memory_kind_person,
		routine: m.memory_kind_routine,
		goal: m.memory_kind_goal,
		fact: m.memory_kind_fact,
	};
	const STRENGTHS: Record<StrengthName, () => string> = {
		faint: m.memory_strength_faint,
		weak: m.memory_strength_weak,
		steady: m.memory_strength_steady,
		strong: m.memory_strength_strong,
		core: m.memory_strength_core,
	};

	let filter = $state<MemoryFilter>('all');
	let query = $state('');
	let gone = $state<string[]>([]);
	const memories = $derived(data.memories.filter((memory) => !gone.includes(memory.id)));
	const counts = $derived(memoryCounts(memories));
	const topics = $derived(memoryTopics(memories, filter, query));
	let opened = $state<Record<string, boolean>>({});
	const date = (at: string) => dayMonth(new Date(at), language);
	const named = (value: number) => {
		const found = strength(value);
		return { level: found.level, name: STRENGTHS[found.name]() };
	};

	let busy = $state<string>();
	let failed = $state<{ id: string; message: string }>();

	async function act(memory: MemoryItem, name: 'edit' | 'confirm' | 'forget', fields: Record<string, string> = {}) {
		busy = memory.id;
		failed = undefined;
		if (name === 'forget') gone = [...gone, memory.id];
		const result = await pageAction(PATH, name, { id: memory.id, ...fields });
		busy = undefined;
		if (!result.ok) {
			gone = gone.filter((id) => id !== memory.id);
			if (result.field === 'content') failed = { id: memory.id, message: result.message };
			else if (result.message) toasts.show({ kind: 'error', title: m.action_failed(), detail: result.message });
			return;
		}
		if (name !== 'confirm') toasts.show({ kind: 'success', title: name === 'forget' ? m.memory_forgotten() : m.memory_corrected() });
		await invalidate('app:memory');
		gone = gone.filter((id) => id !== memory.id);
	}

	// Who learned it: the built-in member's mascot and name, or a custom member's name.
	function learner(type: string | null | undefined): { agent?: ReturnType<typeof crewOf>; name: string } | undefined {
		if (!type) return undefined;
		const agent = crewOf(type);
		const name = data.crew.rows.find((row) => agent && row.agent === agent)?.name ?? type.charAt(0).toUpperCase() + type.slice(1).replaceAll('_', ' ');
		return { agent, name };
	}
	const learnedAny = $derived(memories.some((memory) => memory.learned_by));
	const metaOf = (memory: MemoryItem) => {
		const used = memory.uses > 0 && memory.last_used_at ? m.memories_used({ count: memory.uses, date: date(memory.last_used_at) }) : m.memory_never_used();
		const by = learner(memory.learned_by);
		return by ? `${used} · ${m.memory_learned_by({ name: by.name })}` : used;
	};

	// Learning and Budget: switched at once, put back if saving fails.
	let settings = $derived<MemorySettingsValues | null>(data.settings);
	async function changeSettings(change: MemorySettingsChange) {
		if (!settings) return;
		const before = settings;
		settings = { ...settings, ...change };
		const fields = Object.fromEntries(Object.entries(change).map(([key, value]) => [key, String(value)]));
		const result = await pageAction(PATH, 'settings', fields);
		if (result.ok) return void (settings = result.data.settings as MemorySettingsValues);
		settings = before;
		toasts.show({ kind: 'error', title: m.memory_settings_saved_failed(), detail: result.message || undefined });
	}

	// The profile: saved in the person's words; clearing it offers Undo.
	let profile = $derived<MemoryProfileData | null>(data.profile);
	let profileBusy = $state(false);
	let profileError = $state<string>();
	const initials = $derived(
		data.person.name
			.split(/\s+/)
			.filter(Boolean)
			.slice(0, 2)
			.map((word) => word[0]!.toUpperCase())
			.join(''),
	);
	async function saveProfile(text: string, after?: () => void) {
		profileBusy = true;
		profileError = undefined;
		const result = await pageAction(PATH, 'profile', { text });
		profileBusy = false;
		if (!result.ok) return void (profileError = result.message || m.action_failed());
		profile = result.data.profile as MemoryProfileData;
		after?.();
		if (settings) settings = { ...settings, profile_used: profile.tokens };
	}
	function clearProfile() {
		const before = profile?.text ?? '';
		void saveProfile('', () => toasts.show({ kind: 'success', title: m.memory_profile_cleared(), action: { label: m.memory_undo(), onclick: () => void saveProfile(before) } }));
	}

	// Telling Nomi: a chat with it, where the crew keeps or drops the memory and says so.
	let telling = $state(false);
	async function tell(text: string) {
		telling = true;
		const started = await startChat(text, null);
		telling = false;
		if (started.kind === 'opened') return void goto(started.href);
		if (started.kind === 'failed') toasts.show({ kind: 'error', title: m.memory_tell_failed(), detail: started.message });
		else if (started.kind === 'offline') toasts.show({ kind: 'error', title: m.memory_tell_failed() });
	}

	// On a phone (Flow / Mobile — You & Settings, steps 16 and 17): the profile every crew member
	// reads, then what the crew learned by kind, each a row into its memories (`?topic=`); then the
	// learning switches, the budgets (picked from a sheet) and Export as a row. A memory's ⋯ opens
	// Correct (its own screen), Still right and Forget, which offers Undo before it's sent.
	const guess = () => data.phone;
	const phone = phoneView(guess());
	const screen = phoneScreen();
	const KIND_ICONS: Record<string, typeof Heart> = { preference: Heart, person: Users, routine: Repeat, goal: Goal, fact: Lightbulb };
	const topicKind = $derived(page.url.searchParams.get('topic'));
	const allTopics = $derived(memoryTopics(memories, 'all', ''));
	const shownTopic = $derived(allTopics.find((topic) => topic.kind === topicKind));
	let writing = $state<{ kind: 'profile' } | { kind: 'memory'; memory: MemoryItem } | null>(null);
	let draft = $state('');
	let tools = $state<MemoryItem | null>(null);
	let budgetSheet = $state<'memory_budget' | 'profile_budget' | null>(null);
	$effect(() => {
		screen.inside = phone.current && writing !== null;
		return () => (screen.inside = false);
	});
	const BUDGETS = { memory_budget: [1_000, 2_000, 5_000, 10_000, 20_000, 50_000, 100_000], profile_budget: [250, 500, 1_000, 2_000, 4_000] } as const;
	const tokens = (value: number) => value.toLocaleString(language === 'id' ? 'id-ID' : 'en-US');
	const budgetChoices = (key: 'memory_budget' | 'profile_budget') => [...new Set([...BUDGETS[key], settings?.[key] ?? 0])].filter(Boolean).sort((a, b) => a - b);

	function write(next: NonNullable<typeof writing>) {
		draft = next.kind === 'profile' ? (profile?.text ?? '') : next.memory.content;
		profileError = undefined;
		failed = undefined;
		writing = next;
	}

	async function saveWriting() {
		if (!writing) return;
		if (writing.kind === 'profile') return void saveProfile(draft.trim(), () => ((writing = null), toasts.show({ kind: 'success', title: m.memory_profile_saved() })));
		const memory = writing.memory;
		await act(memory, 'edit', { content: draft.trim() });
		if (failed?.id !== memory.id) writing = null;
	}

	/** Forget, after a moment to take it back: it's hidden at once and sent once Undo has gone. */
	function forgetLater(memory: MemoryItem) {
		gone = [...gone, memory.id];
		let undone = false;
		const send = setTimeout(async () => {
			if (undone) return;
			const result = await pageAction(PATH, 'forget', { id: memory.id });
			if (!result.ok) {
				gone = gone.filter((id) => id !== memory.id);
				if (result.message) toasts.show({ kind: 'error', title: m.action_failed(), detail: result.message });
				return;
			}
			await invalidate('app:memory');
			gone = gone.filter((id) => id !== memory.id);
		}, 6_000);
		toasts.show({
			kind: 'success',
			title: m.memory_forgotten(),
			duration: 6_000,
			action: {
				label: m.memory_undo(),
				onclick: () => {
					undone = true;
					clearTimeout(send);
					gone = gone.filter((id) => id !== memory.id);
				},
			},
		});
	}
</script>

<svelte:head><title>{m.settings_memory()} · {m.app_name()}</title></svelte:head>

{#if phone.current}
	<div class="h-full overflow-y-auto">
		{#if writing}
			<PhoneFormScreen
				backLabel={writing.kind === 'profile' || !shownTopic ? m.phone_memory_back() : (KINDS[shownTopic.kind] ?? (() => shownTopic.kind))()}
				onback={() => (writing = null)}
				title={writing.kind === 'profile' ? m.memory_profile() : m.memory_edit()}
				sub={writing.kind === 'profile' ? m.phone_memory_profile_sub() : m.phone_memory_correct_sub()}
				onsubmit={saveWriting}
			>
				<TextArea
					label={writing.kind === 'profile' ? m.memory_profile_field() : m.memory_field()}
					bind:value={draft}
					rows={writing.kind === 'profile' ? 8 : 4}
					error={writing.kind === 'profile' ? profileError : failed?.id === writing.memory.id ? failed.message : undefined}
					helper={writing.kind === 'profile' && settings ? m.memory_profile_size({ used: tokens(profile?.tokens ?? 0), budget: tokens(settings.profile_budget) }) : undefined}
				/>
				{#snippet action()}
					<PhoneButton
						label={m.memory_save()}
						disabled={profileBusy || !!busy || (writing?.kind === 'memory' && !draft.trim()) || draft.trim() === (writing?.kind === 'profile' ? (profile?.text ?? '') : writing?.memory.content)}
						onclick={saveWriting}
					/>
				{/snippet}
			</PhoneFormScreen>
		{:else if shownTopic}
			<PhoneBackBar label={m.phone_memory_back()} href={PATH} />
			<LargeTitle title={(KINDS[shownTopic.kind] ?? (() => shownTopic.kind))()} />
			<p class="-mt-1 px-5 pb-3.5 text-phone-sm text-text-3">{shownTopic.memories.length === 1 ? m.memory_count_one() : m.memory_count({ count: shownTopic.memories.length })}</p>
			<div class="flex flex-col gap-2.5 px-4 pb-8">
				{#each shownTopic.memories as memory (memory.id)}
					<PhoneMemoryCard
						content={memory.content}
						strength={named(memory.weight)}
						meta={metaOf(memory)}
						check={memory.needs_check ? m.memory_check() : undefined}
						more={m.phone_memory_more()}
						busy={busy === memory.id}
						onmore={() => (tools = memory)}
					/>
				{/each}
			</div>
		{:else}
			<PhoneBackBar label={m.nav_you()} href="/settings" />
			<LargeTitle title={m.settings_memory()} />
			<div class="flex flex-col gap-5 px-4 pt-1 pb-8">
				{#if profile}
					<section aria-labelledby="phone-about" class="flex flex-col gap-2">
						<div class="flex items-center gap-2 px-1">
							<h2 id="phone-about" class="min-w-0 flex-1 font-mono text-label tracking-label text-text-3 uppercase">{m.memory_you()}</h2>
							{#if settings}<span class="shrink-0 font-mono text-label text-text-3">{m.memory_profile_size({ used: tokens(profile.tokens), budget: tokens(settings.profile_budget) })}</span>{/if}
						</div>
						<PhoneMemoryProfile
							text={profile.text}
							empty={settings && !settings.keep_profile ? m.memory_profile_off() : m.memory_profile_empty()}
							updated={profile.updated_at ? m.memory_profile_updated({ date: date(profile.updated_at) }) : null}
							edit={m.phone_memory_edit()}
							editLabel={m.memory_profile_edit()}
							onedit={() => write({ kind: 'profile' })}
						/>
					</section>
				{/if}
				{#if allTopics.length}
					<PhoneSettingsGroup
						label={m.phone_memory_learned()}
						note={String(counts.all)}
						rows={allTopics.map((topic) => ({
							key: topic.kind,
							icon: KIND_ICONS[topic.kind] ?? Sparkles,
							label: (KINDS[topic.kind] ?? (() => topic.kind))(),
							meta: String(topic.memories.length),
							href: `?topic=${encodeURIComponent(topic.kind)}`,
						}))}
					/>
				{:else}
					<PhoneSettingsGroup label={m.phone_memory_learned()} rows={[{ key: 'none', icon: Brain, label: m.memory_none_title(), sub: m.memory_none_body() }]} />
				{/if}
				{#if settings}
					<PhoneSettingsGroup
						label={m.memory_learning()}
						rows={[
							{ key: 'learn', icon: Sparkles, label: m.memory_learn(), sub: m.memory_learn_body(), toggle: { checked: settings.learn_from_chats, onchange: (on) => void changeSettings({ learn_from_chats: on }) } },
							{ key: 'profile', icon: UserRound, label: m.memory_keep_profile(), sub: m.memory_keep_profile_body(), toggle: { checked: settings.keep_profile, onchange: (on) => void changeSettings({ keep_profile: on }) } },
							{ key: 'search', icon: SearchIcon, label: m.memory_search_chats(), sub: m.memory_search_chats_body(), toggle: { checked: settings.search_past_chats, onchange: (on) => void changeSettings({ search_past_chats: on }) } },
						]}
					/>
					<PhoneSettingsGroup
						label={m.memory_budget()}
						rows={[
							{ key: 'memory', icon: Gauge, label: m.memory_budget_memory(), meta: `${tokens(settings.memory_used)} / ${tokens(settings.memory_budget)} tok`, mono: true, onclick: () => (budgetSheet = 'memory_budget') },
							{ key: 'profile', icon: History, label: m.memory_budget_profile(), meta: `${tokens(settings.profile_used)} / ${tokens(settings.profile_budget)} tok`, mono: true, onclick: () => (budgetSheet = 'profile_budget') },
						]}
					/>
				{:else}
					<SettingNote text={m.memory_settings_unloaded()} />
				{/if}
				<PhoneSettingsGroup rows={[{ key: 'export', icon: Download, label: m.phone_memory_export(), href: data.exportLink.href, download: true }]} />
			</div>
		{/if}
	</div>

	<ActionSheet
		open={tools !== null}
		title={tools?.content ?? ''}
		cancel={m.action_cancel()}
		actions={tools
			? [
					{ label: m.memory_edit(), onclick: () => tools && (write({ kind: 'memory', memory: tools }), (tools = null)) },
					...(tools.needs_check ? [{ label: m.memory_still_right(), onclick: () => tools && (void act(tools, 'confirm'), (tools = null)) }] : []),
					{ label: m.memory_forget(), danger: true, onclick: () => tools && (forgetLater(tools), (tools = null)) },
				]
			: []}
		onclose={() => (tools = null)}
	/>
	<BottomSheet open={budgetSheet !== null} title={budgetSheet === 'profile_budget' ? m.memory_budget_profile() : m.memory_budget_memory()} subtitle={m.phone_memory_budget_sub()} onclose={() => (budgetSheet = null)}>
		{#if budgetSheet && settings}
			{#each budgetChoices(budgetSheet) as choice (choice)}
				<SheetOption
					label="{tokens(choice)} tok"
					chosen={choice === settings[budgetSheet]}
					onclick={() => {
						const key = budgetSheet!;
						budgetSheet = null;
						void changeSettings({ [key]: choice });
					}}
				/>
			{/each}
		{/if}
	</BottomSheet>
{:else}

{#snippet memorySettings()}
	{#if settings}
		<MemorySettings
			values={settings}
			{language}
			labels={{
				learning: m.memory_learning(),
				budget: m.memory_budget(),
				learn: { title: m.memory_learn(), body: m.memory_learn_body() },
				profile: { title: m.memory_keep_profile(), body: m.memory_keep_profile_body() },
				search: { title: m.memory_search_chats(), body: m.memory_search_chats_body() },
				memoryBudget: m.memory_budget_memory(),
				profileBudget: m.memory_budget_profile(),
				unit: 'tok',
				used: (used, budget) => m.memory_budget_used({ used, budget }),
				outOfRange: (min, max) => m.memory_budget_range({ min, max }),
			}}
			onchange={changeSettings}
		/>
	{:else}
		<div class="px-5 py-3.5"><SettingNote text={m.memory_settings_unloaded()} /></div>
	{/if}
{/snippet}

<div class="flex h-full">
	<div class="flex min-w-0 flex-1 flex-col">
		<div class="min-h-0 flex-1 overflow-y-auto">
			<div class="flex max-w-220 flex-col gap-4 px-4 pt-6 pb-4 lg:px-8">
				<header class="flex flex-wrap items-end justify-between gap-x-6 gap-y-3">
					<div class="flex max-w-165 min-w-0 flex-1 basis-80 flex-col gap-1.5">
						<h1 class="text-display font-medium tracking-display text-text">{m.settings_memory()}</h1>
						<p class="text-body text-text-2">{m.memory_intro()}</p>
					</div>
					<div class="w-60 max-w-full"><Search placeholder={m.memory_search()} bind:value={query} /></div>
				</header>

				{#if profile}
					<MemoryProfile
						labels={{
							title: m.memory_profile(),
							section: m.memory_you(),
							edit: m.memory_profile_edit(),
							clear: m.memory_profile_clear(),
							save: m.memory_save(),
							cancel: m.memory_cancel(),
							field: m.memory_profile_field(),
							empty: settings && !settings.keep_profile ? m.memory_profile_off() : m.memory_profile_empty(),
							tooLong: m.memory_profile_too_long(),
						}}
						{initials}
						text={profile.text}
						updated={profile.updated_at ? m.memory_profile_updated({ date: date(profile.updated_at) }) : null}
						count={m.memory_profile_size({ used: profile.tokens.toLocaleString(language === 'id' ? 'id-ID' : 'en-US'), budget: (settings?.profile_budget ?? 1000).toLocaleString(language === 'id' ? 'id-ID' : 'en-US') })}
						budget={settings?.profile_budget ?? 1000}
						busy={profileBusy}
						error={profileError}
						onsave={(text) => saveProfile(text, () => toasts.show({ kind: 'success', title: m.memory_profile_saved() }))}
						onclear={clearProfile}
					/>
				{/if}

				<section aria-labelledby="topics-title" class="flex flex-col gap-2">
					<div class="flex flex-wrap items-center justify-between gap-2">
						<span id="topics-title"><SectionLabel label={m.memory_topics()} count={m.memory_topics_count({ topics: counts.topics, memories: counts.all })} /></span>
						<Segmented
							label={m.memory_filter()}
							options={[
								{ value: 'all', label: m.memory_filter_all(), count: counts.all },
								{ value: 'core', label: m.memory_filter_core(), count: counts.core },
								{ value: 'fading', label: m.memory_filter_fading(), count: counts.fading },
							]}
							value={filter}
							onchange={(value) => (filter = value as MemoryFilter)}
						/>
					</div>
					{#if topics.length}
						<MemoryTable
							columns={{ topic: m.memory_col_topic(), knows: m.memory_col_knows(), ...(learnedAny && { learnedBy: m.memory_col_learned_by() }), strength: m.memory_col_strength(), updated: m.memory_col_updated() }}
							note={m.memory_note()}
						>
							{#each topics as topic (topic.kind)}
								<MemoryTopicRow
									name={(KINDS[topic.kind] ?? (() => topic.kind))()}
									count={topic.memories.length === 1 ? m.memory_count_one() : m.memory_count({ count: topic.memories.length })}
									summary={topic.summary}
									learnedBy={learnedAny ? (learner(topLearner(topic.memories)) ?? null) : undefined}
									strength={{ level: topic.strength.level, name: STRENGTHS[topic.strength.name]() }}
									updated={date(topic.updated)}
									fading={topic.fading}
									bind:open={() => opened[topic.kind] ?? (!!query || filter !== 'all'), (open) => (opened = { ...opened, [topic.kind]: open })}
								>
									{#each topic.memories as memory (memory.id)}
										<MemoryEntry
											content={memory.content}
											strength={named(memory.weight)}
											meta={metaOf(memory)}
											check={memory.needs_check ? { question: m.memory_check(), confirm: m.memory_still_right() } : undefined}
											labels={{ edit: m.memory_edit(), forget: m.memory_forget(), save: m.memory_save(), cancel: m.memory_cancel(), field: m.memory_field() }}
											busy={busy === memory.id}
											error={failed?.id === memory.id ? failed.message : undefined}
											onsave={(content) => act(memory, 'edit', { content })}
											onforget={() => act(memory, 'forget')}
											onconfirm={() => act(memory, 'confirm')}
										/>
									{/each}
								</MemoryTopicRow>
							{/each}
						</MemoryTable>
					{:else if counts.all}
						<p class="py-3 text-caption text-text-3">{m.memory_none_match()}</p>
					{:else}
						<EmptyState icon={Brain} title={m.memory_none_title()} body={m.memory_none_body()} />
					{/if}
				</section>

				<div class="overflow-hidden rounded-md outline -outline-offset-1 outline-border xl:hidden">{@render memorySettings()}</div>
			</div>
		</div>
		<div class="max-w-220 shrink-0 px-4 pt-2 pb-4 lg:px-8">
			<ChatComposer placeholder={m.memory_tell()} busy={telling} hints onsend={tell} />
		</div>
	</div>
	<aside aria-label={m.memory_settings_label()} class="hidden w-95 shrink-0 overflow-y-auto border-l border-border xl:block">{@render memorySettings()}</aside>
</div>
{/if}
