<script lang="ts">
	import { onMount } from 'svelte';
	import { Activity, Bot } from '@lucide/svelte';
	import { goto, invalidate } from '$app/navigation';
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { tabId } from '$lib/device';
	import { createLiveLink } from '$lib/live/link';
	import { agentName, applyFrame, eventSentence, liveRows, mascotFor, parseFrame, phaseLabel, runningCommand, shortId, type LiveRow, type LiveState } from '$lib/admin/agents';
	import { dateTime } from '$lib/admin/format';
	import { timeZone } from '$lib/admin/zone.svelte';
	import PageHeader from '$lib/ds/admin/PageHeader.svelte';
	import AdminTable, { type TableColumn, type TableSort } from '$lib/ds/admin/AdminTable.svelte';
	import Status from '$lib/ds/status/Status.svelte';
	import SectionLabel from '$lib/ds/status/SectionLabel.svelte';
	import EmptyState from '$lib/ds/lists/EmptyState.svelte';
	import Mascot from '$lib/ds/agents/Mascot.svelte';
	import Dialog from '$lib/ds/overlays/Dialog.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import LoadFailed from '$lib/admin/ui/LoadFailed.svelte';

	// Live agents (dashboard: view): every agent at work, kept current by the admin stream (relayed
	// at /admin/agents/ws), and the latest events as sentences. The stream replays nothing, so after
	// a reconnect the page reloads what it missed. One agent's steps open from its row.
	let { data } = $props();

	const language = getLocale();
	const zone = timeZone(() => page.data.timeZone as string | null);
	const fresh = (): LiveState => ({ rows: data.agents.ok ? liveRows(data.agents.data) : [], feed: data.events.ok ? data.events.data : [] });
	let live = $state<LiveState>(fresh());
	let link = $state<'connecting' | 'live' | 'reconnecting' | 'stopped'>('connecting');
	let sort = $state<TableSort>({ key: 'started', direction: 'descending' });

	// A reload (after a reconnect) starts again from what the backend says now.
	$effect(() => {
		live = fresh();
	});

	onMount(() => {
		const scheme = location.protocol === 'https:' ? 'wss' : 'ws';
		const stream = createLiveLink(
			{
				opened(resumed) {
					link = 'live';
					if (resumed) void invalidate((url) => url.pathname === '/admin/agents');
				},
				message(raw) {
					const frame = parseFrame(raw);
					if (frame) live = applyFrame(live, frame, new Date().toISOString());
				},
				lost() {
					link = 'stopped';
				},
			},
			() => {
				const socket = new WebSocket(`${scheme}://${location.host}/admin/agents/ws?tab=${tabId()}`);
				socket.addEventListener('close', () => {
					if (link === 'live') link = 'reconnecting';
				});
				return socket;
			},
		);
		return () => stream.close();
	});

	const columns: TableColumn<LiveRow>[] = [
		{ key: 'person', label: m.admin_col_person(), sort: (row) => row.person },
		{ key: 'agent', label: m.admin_col_agent(), sort: (row) => agentName(row.agent_type, row.agent_display_name) },
		{ key: 'doing', label: m.admin_col_doing() },
		{ key: 'chat', label: m.admin_col_chat(), phone: false },
		{ key: 'channel', label: m.admin_col_channel(), phone: false },
		{ key: 'started', label: m.admin_col_started(), sort: (row) => row.started_at, phone: false },
		{ key: 'last', label: m.admin_col_last(), sort: (row) => row.last_activity_at, phone: false },
		{ key: 'steps', label: '' },
	];

	const opened = $derived(data.detail ? live.rows.find((row) => row.agent_session_id === data.detail?.agent) : undefined);
	const openedType = $derived(opened?.agent_type ?? (data.detail?.events.ok ? (data.detail.events.data[0]?.agent_type ?? null) : null));
	const openedName = $derived(agentName(openedType, opened?.agent_display_name ?? null));
	let stepsOpen = $state(false);
	$effect(() => {
		stepsOpen = Boolean(data.detail);
	});

	function closeSteps() {
		const url = new URL(page.url);
		url.searchParams.delete('agent');
		url.searchParams.delete('session');
		void goto(url, { noScroll: true, keepFocus: true, replaceState: true });
	}
</script>

<PageHeader title={m.admin_page_live()} intro={m.admin_live_intro()}>
	{#snippet actions()}
		{#if link === 'live'}
			<Status kind="live" label={m.admin_live_connected()} />
		{:else if link === 'connecting'}
			<Status kind="idle" label={m.admin_live_connecting()} />
		{:else if link === 'reconnecting'}
			<Status kind="attention" label={m.admin_live_reconnecting()} />
		{:else}
			<Status kind="failed" label={m.admin_live_stopped()} />
		{/if}
	{/snippet}
</PageHeader>

{#if data.agents.ok}
	<section aria-label={m.admin_live_working()} class="flex flex-col gap-3">
		<SectionLabel label={m.admin_live_working()} count={live.rows.length} />
		{#if live.rows.length === 0}
			<EmptyState icon={Bot} title={m.admin_live_empty_title()} body={m.admin_live_empty_body()} />
		{:else}
			<AdminTable label={m.admin_live_table()} {columns} rows={live.rows} rowKey={(row) => row.agent_session_id} bind:sort>
				{#snippet cell(row, column)}
					{#if column.key === 'person'}
						<span class="block max-w-60 truncate">{row.person}</span>
					{:else if column.key === 'agent'}
						{@const crew = mascotFor(row.agent_type)}
						<span class="inline-flex items-center gap-2">
							{#if crew}<Mascot agent={crew} size={16} />{/if}
							{agentName(row.agent_type, row.agent_display_name)}
						</span>
					{:else if column.key === 'doing'}
						{@const command = runningCommand(row.current_phase, row.current_phase_detail)}
						<span class={['flex flex-col gap-0.5', row.current_phase === 'waiting' ? 'text-text-3' : 'text-text-2']}>
							{phaseLabel(row.current_phase, row.current_phase_detail)}
							{#if command}<span class="max-w-60 truncate font-mono text-label text-text-3">{command}</span>{/if}
						</span>
					{:else if column.key === 'chat'}
						<span class="font-mono text-label text-text-2" title={row.session_id}>{shortId(row.session_id)}</span>
					{:else if column.key === 'channel'}
						<span class="font-mono text-label text-text-2">{row.channel}</span>
					{:else if column.key === 'started'}
						<span class="font-mono text-label whitespace-nowrap text-text-2">{dateTime(row.started_at, language, zone.value)}</span>
					{:else if column.key === 'last'}
						<span class="font-mono text-label whitespace-nowrap text-text-2">{dateTime(row.last_activity_at, language, zone.value)}</span>
					{:else}
						<Button label={m.admin_live_steps()} variant="ghost" href="?agent={row.agent_session_id}&session={row.session_id}" data-sveltekit-noscroll="" />
					{/if}
				{/snippet}
			</AdminTable>
		{/if}
	</section>

	<section aria-label={m.admin_live_feed()} class="flex flex-col gap-3">
		<SectionLabel label={m.admin_live_feed()} />
		{#if !data.events.ok}
			<LoadFailed title={m.admin_live_feed_failed()} problem={data.events.problem} />
		{:else if live.feed.length === 0}
			<EmptyState icon={Activity} title={m.admin_live_feed()} body={m.admin_live_feed_empty_body()} />
		{:else}
			<ol class="flex flex-col rounded-md outline -outline-offset-1 outline-border">
				{#each live.feed as item (item.id)}
					{@const crew = mascotFor(item.agent_type)}
					<li class="flex items-center gap-3 border-b border-border px-3.5 py-2.25 last:border-b-0">
						<span class="inline-flex size-4 shrink-0">{#if crew}<Mascot agent={crew} size={16} />{/if}</span>
						<span class="min-w-0 flex-1 text-ui text-text-2">{eventSentence(item)}</span>
						<span class="shrink-0 font-mono text-label text-text-3">{dateTime(item.created_at, language, zone.value)}</span>
					</li>
				{/each}
			</ol>
		{/if}
	</section>
{:else}
	<LoadFailed title={m.admin_live_failed()} problem={data.agents.problem} />
{/if}

{#if data.detail}
	<Dialog bind:open={stepsOpen} wide title={m.admin_live_steps_for({ agent: openedName, person: opened?.person ?? shortId(data.detail.session) })} onclose={closeSteps}>
		{#if !data.detail.events.ok}
			<LoadFailed title={m.admin_live_detail_failed()} problem={data.detail.events.problem} />
		{:else if data.detail.events.data.length === 0}
			<p class="text-ui text-text-2">{m.admin_live_detail_empty()}</p>
		{:else}
			<ol class="flex max-h-96 flex-col overflow-y-auto">
				{#each data.detail.events.data as item (item.id)}
					<li class="flex items-start justify-between gap-3 border-b border-border py-2 last:border-b-0">
						<span class="text-ui text-text-2">{eventSentence(item)}</span>
						<span class="shrink-0 font-mono text-label text-text-3">{dateTime(item.created_at, language, zone.value)}</span>
					</li>
				{/each}
			</ol>
		{/if}
		{#snippet actions()}<Button label={m.admin_close()} variant="secondary" onclick={() => (stepsOpen = false)} />{/snippet}
	</Dialog>
{/if}
