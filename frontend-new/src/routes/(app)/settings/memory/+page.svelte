<script lang="ts">
	import { Brain } from '@lucide/svelte';
	import { goto, invalidate } from '$app/navigation';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { pageAction } from '$lib/pageAction';
	import { startChat } from '$lib/chat/start';
	import { dayMonth } from '$lib/settings/profile';
	import { memoryCounts, memoryTopics, strength, type MemoryFilter, type StrengthName } from '$lib/settings/memory';
	import type { MemoryItem } from '$lib/adapters/api';
	import { toasts } from '$lib/ds/overlays/toasts.svelte';
	import Search from '$lib/ds/inputs/Search.svelte';
	import Segmented from '$lib/ds/inputs/Segmented.svelte';
	import SectionLabel from '$lib/ds/status/SectionLabel.svelte';
	import EmptyState from '$lib/ds/lists/EmptyState.svelte';
	import ChatComposer from '$lib/ds/thread/ChatComposer.svelte';
	import MemoryTable from '$lib/ds/settings/MemoryTable.svelte';
	import MemoryTopicRow from '$lib/ds/settings/MemoryTopicRow.svelte';
	import MemoryEntry from '$lib/ds/settings/MemoryEntry.svelte';

	// Settings › Memory & context (App / Settings / Memory hANTE): what the crew remembers, by topic
	// (the backend's kinds), each opening to its memories to correct, confirm or forget; the Core
	// and Fading views and a search; and the composer to tell Nomi something to remember or forget,
	// which starts a chat with it. Left out (AGENTS.md §9, BACKEND_TODO.md): the profile row, who
	// learned each topic, and the Learning, Budget and Storage settings, which the backend doesn't
	// keep; the search sits on the page, not in the top bar.
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
</script>

<svelte:head><title>{m.settings_memory()} · {m.app_name()}</title></svelte:head>

<div class="flex h-full flex-col">
	<div class="min-h-0 flex-1 overflow-y-auto">
		<div class="flex max-w-220 flex-col gap-4 px-4 pt-6 pb-4 lg:px-8">
			<header class="flex flex-wrap items-end justify-between gap-x-6 gap-y-3">
				<div class="flex max-w-165 min-w-0 flex-1 basis-80 flex-col gap-1.5">
					<h1 class="text-display font-medium tracking-display text-text">{m.settings_memory()}</h1>
					<p class="text-body text-text-2">{m.memory_intro()}</p>
				</div>
				<div class="w-60 max-w-full"><Search placeholder={m.memory_search()} bind:value={query} /></div>
			</header>

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
					<MemoryTable columns={{ topic: m.memory_col_topic(), knows: m.memory_col_knows(), strength: m.memory_col_strength(), updated: m.memory_col_updated() }} note={m.memory_note()}>
						{#each topics as topic (topic.kind)}
							<MemoryTopicRow
								name={(KINDS[topic.kind] ?? (() => topic.kind))()}
								count={topic.memories.length === 1 ? m.memory_count_one() : m.memory_count({ count: topic.memories.length })}
								summary={topic.summary}
								strength={{ level: topic.strength.level, name: STRENGTHS[topic.strength.name]() }}
								updated={date(topic.updated)}
								fading={topic.fading}
								bind:open={() => opened[topic.kind] ?? (!!query || filter !== 'all'), (open) => (opened = { ...opened, [topic.kind]: open })}
							>
								{#each topic.memories as memory (memory.id)}
									<MemoryEntry
										content={memory.content}
										strength={named(memory.weight)}
										meta={memory.uses > 0 && memory.last_used_at ? m.memories_used({ count: memory.uses, date: date(memory.last_used_at) }) : m.memory_never_used()}
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
		</div>
	</div>
	<div class="max-w-220 shrink-0 px-4 pt-2 pb-4 lg:px-8">
		<ChatComposer placeholder={m.memory_tell()} busy={telling} hints onsend={tell} />
	</div>
</div>
