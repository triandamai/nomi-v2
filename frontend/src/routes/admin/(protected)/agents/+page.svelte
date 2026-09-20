<script lang="ts">
	import { onMount } from 'svelte';
	import DataTable from '$lib/components/m3/DataTable.svelte';
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
		const who = item.agent_display_name ?? (item.agent_type ? agentTypeFallbackLabel(item.agent_type) : 'An agent');
		return eventFeedLine(item.event_type, who, item.session_id, item.tool_name, item.is_error);
	}

	function rowPhaseLabel(row: Row): string {
		return sharedPhaseLabel(row.current_phase, row.current_phase_detail);
	}

	const columns = [
		{ key: 'user_label', label: 'User', sortable: true },
		{ key: 'agent_display_name', label: 'Agent', sortable: true },
		{ key: 'session_id', label: 'Session', sortable: true },
		{ key: 'channel', label: 'Channel', sortable: true },
		{ key: 'current_phase', label: 'Status', sortable: true },
		{ key: 'started_at', label: 'Started', sortable: true },
		{ key: 'last_activity_at', label: 'Last Activity', sortable: true },
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

<h1 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">Command center</h1>
<p class="md-body-large mt-2" style="color: var(--md-sys-color-on-surface-variant)">
	Every active agent, live.
</p>

<div class="mt-6">
	{#if sortedRows.length === 0}
		<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">
			No agents are currently running.
		</p>
	{:else}
		<DataTable {columns} bind:sortKey bind:sortDirection>
			{#each sortedRows as row (row.agent_session_id)}
				<tr onclick={() => openDrillDown(row)} style="cursor: pointer;">
					<td>{row.user_label}</td>
					<td>{row.agent_display_name}</td>
					<td title={row.session_id}>{shortSessionId(row.session_id)}</td>
					<td>{row.channel}</td>
					<td>{rowPhaseLabel(row)}</td>
					<td>{new Date(row.started_at).toLocaleString()}</td>
					<td>{new Date(row.last_activity_at).toLocaleString()}</td>
				</tr>
			{/each}
		</DataTable>
	{/if}
</div>

<div class="mt-8">
	<h2 class="md-title-large" style="color: var(--md-sys-color-on-surface)">Activity</h2>
	<div class="mt-2 flex flex-col gap-1 max-h-96 overflow-y-auto">
		{#each feed as item (item.id)}
			<div class="md-body-medium flex items-center justify-between px-2 py-1" style="border-bottom: 1px solid var(--md-sys-color-outline-variant)">
				<span style="color: var(--md-sys-color-on-surface)">{feedLine(item)}</span>
				<span class="md-body-small" style="color: var(--md-sys-color-on-surface-variant)">{new Date(item.created_at).toLocaleTimeString()}</span>
			</div>
		{/each}
	</div>
</div>

<SideSheet bind:open={drillDownOpen}>
	{#snippet children()}
		{#if drillDownRow}
			<h2 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface); margin: 0 0 4px;">
				{drillDownRow.agent_display_name}
			</h2>
			<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant); margin: 0 0 16px;" title={drillDownRow.session_id}>
				Session {drillDownRow.session_id ? shortSessionId(drillDownRow.session_id) : '(unknown until a live event arrives)'} · {rowPhaseLabel(drillDownRow)}
			</p>
			{#if drillDownLoading}
				<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">Loading…</p>
			{:else if drillDownEvents.length === 0}
				<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">No recent activity for this agent.</p>
			{:else}
				{#each drillDownEvents as item (item.id)}
					<div style="padding: 8px 0; border-bottom: 1px solid var(--md-sys-color-outline-variant);">
						<p class="md-body-large" style="color: var(--md-sys-color-on-surface)">{feedLine(item)}</p>
						<p class="md-body-small" style="color: var(--md-sys-color-on-surface-variant)">{new Date(item.created_at).toLocaleString()}</p>
					</div>
				{/each}
			{/if}
		{/if}
	{/snippet}
</SideSheet>
