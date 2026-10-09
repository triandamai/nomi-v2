<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { onMount } from 'svelte';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import DataTable from '$lib/components/m3/DataTable.svelte';
	import ListItem from '$lib/components/m3/ListItem.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import SideSheet from '$lib/components/m3/SideSheet.svelte';
	import { agentTypeFallbackLabel, eventFeedLine, phaseLabel as sharedPhaseLabel, shortSessionId } from '$lib/agentLabels';
	import type { AdminStreamFrame, AgentEventItem } from '$lib/types';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	type Row = {
		agent_session_id: string;
		// A single agent_display_name (e.g. "Planning") can be running in more than one session
		// at once — the Session column below is what actually disambiguates one row from another.
		session_id: string;
		user_label: string;
		agent_type: string;
		agent_display_name: string;
		channel: string;
		current_phase: string;
		current_phase_detail: string | null;
		started_at: string;
		last_activity_at: string;
	};

	let rows = $state<Row[]>(
		data.agents.users.flatMap((group) =>
			group.agents.map((agent) => ({
				agent_session_id: agent.agent_session_id,
				session_id: agent.session_id,
				user_label: group.label,
				agent_type: agent.agent_type,
				agent_display_name: agent.agent_display_name,
				channel: agent.channel,
				current_phase: agent.current_phase,
				current_phase_detail: agent.current_phase_detail,
				started_at: agent.started_at,
				last_activity_at: agent.last_activity_at,
			})),
		),
	);

	let feed = $state<AgentEventItem[]>(data.events);

	function feedLine(item: AgentEventItem): string {
		const who = item.agent_display_name ?? (item.agent_type ? agentTypeFallbackLabel(item.agent_type) : m.agent_an_agent());
		return eventFeedLine(item.event_type, who, item.session_id, item.tool_name, item.is_error);
	}

	function rowPhaseLabel(row: Row): string {
		return sharedPhaseLabel(row.current_phase, row.current_phase_detail);
	}

	const columns = [
		{ key: 'user_label', label: m.live_user(), sortable: true },
		{ key: 'agent_display_name', label: m.live_agent(), sortable: true },
		{ key: 'session_id', label: m.live_session(), sortable: true },
		{ key: 'channel', label: m.live_channel(), sortable: true },
		{ key: 'current_phase', label: m.live_status(), sortable: true },
		{ key: 'started_at', label: m.live_started(), sortable: true },
		{ key: 'last_activity_at', label: m.live_last_activity(), sortable: true },
	];

	let sortKey = $state<string | undefined>('user_label');
	let sortDirection = $state<'asc' | 'desc'>('asc');

	const sortedRows = $derived.by(() => {
		if (!sortKey) return rows;
		const key = sortKey as keyof Row;
		const direction = sortDirection === 'asc' ? 1 : -1;
		return [...rows].sort((a, b) => {
			const av = a[key] ?? '';
			const bv = b[key] ?? '';
			if (av < bv) return -1 * direction;
			if (av > bv) return 1 * direction;
			return 0;
		});
	});

	// Paged here: the whole list streams in live.
	const PAGE_SIZES = [10, 25, 50];
	let page = $state(1);
	let pageSize = $state(10);
	const pageCount = $derived(Math.max(1, Math.ceil(sortedRows.length / pageSize)));
	$effect(() => {
		if (page > pageCount) page = pageCount;
	});
	const pageRows = $derived(sortedRows.slice((page - 1) * pageSize, page * pageSize));
	function changePageSize(size: number) {
		page = Math.floor(((page - 1) * pageSize) / size) + 1;
		pageSize = size;
	}

	const TERMINAL_CLOSE_CODES = new Set([4401, 4404]);
	const INITIAL_RETRY_DELAY_MS = 1000;
	const MAX_RETRY_DELAY_MS = 30000;
	const MAX_FEED_ITEMS = 200;

	function pushFeedItem(item: Omit<AgentEventItem, 'id' | 'created_at'>) {
		feed = [{ id: crypto.randomUUID(), created_at: new Date().toISOString(), ...item }, ...feed].slice(0, MAX_FEED_ITEMS);
	}

	function applyFrame(frame: AdminStreamFrame) {
		if (frame.kind === 'AgentSessionStarted') {
			rows = [
				...rows,
				{
					agent_session_id: frame.agent_session_id,
					session_id: frame.session_id,
					user_label: frame.sender_label,
					agent_type: frame.agent_type,
					agent_display_name: frame.agent_display_name,
					channel: frame.channel,
					current_phase: 'waiting',
					current_phase_detail: null,
					started_at: new Date().toISOString(),
					last_activity_at: new Date().toISOString(),
				},
			];
			pushFeedItem({
				session_id: frame.session_id,
				agent_session_id: frame.agent_session_id,
				agent_type: frame.agent_type,
				agent_display_name: frame.agent_display_name,
				event_type: 'AgentSpawned',
				tool_name: null,
				is_error: null,
			});
		} else if (frame.kind === 'AgentSessionEnded') {
			const ended = rows.find((r) => r.agent_session_id === frame.agent_session_id);
			rows = rows.filter((r) => r.agent_session_id !== frame.agent_session_id);
			pushFeedItem({
				session_id: ended?.session_id ?? frame.session_id,
				agent_session_id: frame.agent_session_id,
				agent_type: ended?.agent_type ?? null,
				agent_display_name: ended?.agent_display_name ?? null,
				event_type: frame.reason === 'cancelled' ? 'AgentCancelled' : frame.reason === 'expired' ? 'AgentExpired' : 'AgentCompleted',
				tool_name: null,
				is_error: null,
			});
		} else if (frame.kind === 'AgentPhaseChanged') {
			rows = rows.map((r) =>
				r.agent_session_id === frame.agent_session_id
					? { ...r, current_phase: frame.phase, current_phase_detail: frame.detail, last_activity_at: new Date().toISOString() }
					: r,
			);
			const source = rows.find((r) => r.agent_session_id === frame.agent_session_id);
			if (frame.phase === 'calling_tool' && frame.detail) {
				pushFeedItem({
					session_id: frame.session_id,
					agent_session_id: frame.agent_session_id,
					agent_type: source?.agent_type ?? null,
					agent_display_name: source?.agent_display_name ?? null,
					event_type: 'ToolCalled',
					tool_name: frame.detail,
					is_error: null,
				});
			} else if (frame.phase === 'writing_reply') {
				pushFeedItem({
					session_id: frame.session_id,
					agent_session_id: frame.agent_session_id,
					agent_type: source?.agent_type ?? null,
					agent_display_name: source?.agent_display_name ?? null,
					event_type: 'AgentFinalizing',
					tool_name: null,
					is_error: null,
				});
			}
		}
	}

	onMount(() => {
		let socket: WebSocket | undefined;
		let retryDelay = INITIAL_RETRY_DELAY_MS;
		let retryTimeout: ReturnType<typeof setTimeout> | undefined;
		let intentionallyClosed = false;

		function connect() {
			socket = new WebSocket('/admin/agents/ws');

			socket.addEventListener('open', () => {
				retryDelay = INITIAL_RETRY_DELAY_MS;
			});

			socket.addEventListener('message', (event) => {
				let frame: AdminStreamFrame;
				try {
					frame = JSON.parse(event.data);
				} catch {
					return;
				}
				applyFrame(frame);
			});

			socket.addEventListener('close', (event) => {
				if (intentionallyClosed) return;
				if (TERMINAL_CLOSE_CODES.has(event.code)) return; // 401/404 — not an admin, or the route vanished; don't retry
				const jitter = Math.random() * 250;
				retryTimeout = setTimeout(connect, retryDelay + jitter);
				retryDelay = Math.min(retryDelay * 2, MAX_RETRY_DELAY_MS);
			});
		}

		connect();

		return () => {
			intentionallyClosed = true;
			clearTimeout(retryTimeout);
			socket?.close();
		};
	});

	let drillDownOpen = $state(false);
	let drillDownRow = $state<Row | null>(null);
	let drillDownEvents = $state<AgentEventItem[]>([]);
	let drillDownLoading = $state(false);

	async function openDrillDown(row: Row) {
		drillDownRow = row;
		drillDownOpen = true;
		drillDownLoading = true;
		try {
			const response = await fetch(`/admin/agents/${row.agent_session_id}/events?sessionId=${row.session_id}`);
			drillDownEvents = response.ok ? await response.json() : [];
		} finally {
			drillDownLoading = false;
		}
	}
</script>

<PageHeader title={m.admin_live_agents()} lede={m.live_lede()} agent="supervisor" />

<section aria-labelledby="running-title">
	<h2 id="running-title" class="section-label">{m.live_working_now({ count: sortedRows.length })}</h2>
	{#if sortedRows.length === 0}
		<div class="quiet">
			<AgentShape agent="nomi" face size={56} />
			<p class="quiet__text"><strong>{m.live_quiet_title()}</strong> {m.live_quiet()}</p>
		</div>
	{:else}
		<DataTable
			card
			{columns}
			bind:sortKey
			bind:sortDirection
			bind:page
			{pageSize}
			totalItems={sortedRows.length}
			pageSizeOptions={PAGE_SIZES}
			onPageSizeChange={changePageSize}
		>
			{#each pageRows as row (row.agent_session_id)}
				<tr onclick={() => openDrillDown(row)} style="cursor: pointer;">
					<td>{row.user_label}</td>
					<td>
						<span class="agent">
							<AgentShape agent={row.agent_type} size={28} working={row.current_phase !== 'waiting'} />
							{row.agent_display_name}
						</span>
					</td>
					<td class="mono" title={row.session_id}>{shortSessionId(row.session_id)}</td>
					<td>{row.channel}</td>
					<td><span class="phase" data-phase={row.current_phase}>{rowPhaseLabel(row)}</span></td>
					<td class="mono">{new Date(row.started_at).toLocaleString(getLocale())}</td>
					<td class="mono">{new Date(row.last_activity_at).toLocaleString(getLocale())}</td>
				</tr>
			{/each}
			{#snippet list()}
				{#each pageRows as row (row.agent_session_id)}
					<ListItem
						headline={row.agent_display_name}
						supportingText={`${row.user_label} · ${rowPhaseLabel(row)} · ${new Date(row.last_activity_at).toLocaleTimeString(getLocale())}`}
						onclick={() => openDrillDown(row)}
					>
						{#snippet leading()}
							<AgentShape agent={row.agent_type} size={36} working={row.current_phase !== 'waiting'} />
						{/snippet}
						{#snippet trailing()}
							<span class="mono">{row.channel}</span>
						{/snippet}
					</ListItem>
				{/each}
			{/snippet}
		</DataTable>
	{/if}
</section>

<section aria-labelledby="activity-title">
	<h2 id="activity-title" class="section-label">{m.live_activity()}</h2>
	{#if feed.length === 0}
		<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">{m.live_nothing()}</p>
	{:else}
		<ol class="feed">
			{#each feed as item (item.id)}
				<li class="feed__row">
					<AgentShape agent={item.agent_type ?? item.agent_display_name ?? 'nomi'} size={22} />
					<span class="feed__text">{feedLine(item)}</span>
					<time class="feed__time" datetime={item.created_at}>{new Date(item.created_at).toLocaleTimeString(getLocale())}</time>
				</li>
			{/each}
		</ol>
	{/if}
</section>

<SideSheet bind:open={drillDownOpen}>
	{#snippet children()}
		{#if drillDownRow}
			<h2 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface); margin: 0 0 4px;">
				{drillDownRow.agent_display_name}
			</h2>
			<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant); margin: 0 0 16px;" title={drillDownRow.session_id}>
				{m.live_session_line({ session: drillDownRow.session_id ? shortSessionId(drillDownRow.session_id) : m.live_session_unknown(), phase: rowPhaseLabel(drillDownRow) })}
			</p>
			{#if drillDownLoading}
				<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">{m.common_loading()}</p>
			{:else if drillDownEvents.length === 0}
				<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">{m.live_no_recent()}</p>
			{:else}
				{#each drillDownEvents as item (item.id)}
					<div style="padding: 8px 0; border-bottom: 1px solid var(--md-sys-color-outline-variant);">
						<p class="md-body-large" style="color: var(--md-sys-color-on-surface)">{feedLine(item)}</p>
						<p class="md-body-small" style="color: var(--md-sys-color-on-surface-variant)">{new Date(item.created_at).toLocaleString(getLocale())}</p>
					</div>
				{/each}
			{/if}
		{/if}
	{/snippet}
</SideSheet>

<style>
	.section-label {
		margin: 0 4px 12px;
		color: var(--md-sys-color-on-surface-variant);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.6875rem;
		font-weight: 400;
		letter-spacing: 0.08em;
		text-transform: uppercase;
	}
	.quiet {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 20px 24px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
	}
	.quiet__text {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		line-height: 1.5;
	}
	.quiet__text strong {
		color: var(--md-sys-color-on-surface);
	}
	.agent {
		display: inline-flex;
		align-items: center;
		gap: 10px;
		font-weight: 600;
	}
	.mono {
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.8125rem;
		font-variant-numeric: tabular-nums;
	}
	.phase {
		display: inline-flex;
		padding: 3px 12px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
		font-size: 0.8125rem;
		font-weight: 650;
	}
	.phase[data-phase='waiting'] {
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface-variant);
	}
	.feed {
		display: flex;
		flex-direction: column;
		max-height: 28rem;
		margin: 0;
		padding: 6px 0;
		overflow-y: auto;
		list-style: none;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
	}
	.feed__row {
		display: grid;
		grid-template-columns: auto minmax(0, 1fr) auto;
		align-items: center;
		gap: 12px;
		padding: 10px 20px;
	}
	.feed__row + .feed__row {
		border-top: 1px solid var(--md-sys-color-outline-variant);
	}
	.feed__text {
		color: var(--md-sys-color-on-surface);
		overflow-wrap: anywhere;
	}
	.feed__time {
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
		color: var(--md-sys-color-on-surface-variant);
		font-variant-numeric: tabular-nums;
	}
</style>
