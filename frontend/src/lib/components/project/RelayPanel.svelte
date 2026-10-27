<script lang="ts">
	// A Relay project's backend, for its admins: per environment, its tables (columns, row-level
	// security, policies), how the app reaches it, its buckets, live tables, functions and
	// sign-in, and a SQL console that runs as the project's owner.
	import { m } from '$lib/paraglide/messages';
	import Button from '$lib/components/m3/Button.svelte';
	import IconCopy from '$lib/components/icons/IconCopy.svelte';
	import QueryResultView from '$lib/components/project/QueryResultView.svelte';
	import { projectApi } from '$lib/projectApi';
	import type { DatabaseSource, QueryResult, RelayOverview, RelayRule, RelayTable } from '$lib/types';

	let { projectId, published }: { projectId: string; published: boolean } = $props();

	type Relay = Extract<RelayOverview, { backend: 'relay' }>;

	const ROWS_TO_BROWSE = 100;
	const RULES: Record<RelayRule, () => string> = {
		anyone: m.relay_rule_anyone,
		signed_in: m.relay_rule_signed_in,
		owner: m.relay_rule_owner,
		service: m.relay_rule_service,
	};

	let source = $state<DatabaseSource>('development');
	let relay = $state<Relay | null>(null);
	let loadError = $state<string | null>(null);
	let selected = $state<RelayTable | null>(null);
	let sql = $state('');
	let running = $state(false);
	let result = $state<QueryResult | null>(null);
	let queryError = $state<string | null>(null);
	let copied = $state<string | null>(null);

	async function load() {
		loadError = null;
		try {
			const answer = await projectApi<RelayOverview>(projectId, `relay?source=${source}`);
			relay = answer.backend === 'relay' ? answer : null;
			if (selected) selected = relay?.tables.find((t) => t.name === selected?.name) ?? null;
		} catch (e) {
			loadError = (e as Error).message;
		}
	}

	$effect(() => {
		void source;
		relay = null;
		selected = null;
		result = null;
		void load();
	});

	async function run() {
		if (!sql.trim()) return;
		running = true;
		queryError = null;
		try {
			result = await projectApi<QueryResult>(projectId, 'relay/query', { method: 'POST', body: { sql, source } });
			// A write may have changed the tables themselves.
			if (result.rows_affected !== null) void load();
		} catch (e) {
			queryError = (e as Error).message;
			result = null;
		} finally {
			running = false;
		}
	}

	function browse(table: RelayTable) {
		selected = table;
		sql = `SELECT * FROM "${table.name.replace(/"/g, '""')}" LIMIT ${ROWS_TO_BROWSE}`;
		void run();
	}

	async function copy(what: string, text: string) {
		await navigator.clipboard.writeText(text).catch(() => {});
		copied = what;
		setTimeout(() => (copied = null), 1500);
	}

	function keydown(event: KeyboardEvent) {
		if ((event.metaKey || event.ctrlKey) && event.key === 'Enter') {
			event.preventDefault();
			void run();
		}
	}

	const functions = $derived(Object.entries(relay?.functions?.functions ?? {}));
</script>

<div class="relay">
	<div class="sources" role="tablist" aria-label={m.pj_db_source()}>
		<button type="button" role="tab" class="source" aria-selected={source === 'development'} onclick={() => (source = 'development')}>{m.pj_db_development()}</button>
		<button type="button" role="tab" class="source" aria-selected={source === 'production'} disabled={!published} onclick={() => (source = 'production')}>{m.pj_db_production()}</button>
	</div>
	<p class="hint">{source === 'development' ? m.relay_development_hint() : m.relay_production_hint()}</p>

	{#if loadError}
		<p class="error" role="alert">{loadError}</p>
	{:else if !relay}
		<p class="muted">{m.pj_loading()}</p>
	{:else}
		<div class="layout">
			<aside class="side">
				<h3 class="section-title">{m.pj_db_tables()}</h3>
				{#if relay.tables.length === 0}
					<p class="muted small">{m.pj_db_no_tables()}</p>
				{:else}
					<ul class="list">
						{#each relay.tables as table (table.name)}
							<li>
								<button type="button" class="table" aria-current={selected?.name === table.name} onclick={() => browse(table)}>
									<span class="table__name">{table.name}</span>
									{#if !table.rls_enabled}<span class="warn small">{m.relay_rls_off_short()}</span>{/if}
								</button>
							</li>
						{/each}
					</ul>
				{/if}

				<h3 class="section-title">{m.relay_access()}</h3>
				<p class="muted small">{m.relay_access_hint()}</p>
				<div class="key">
					<span class="key__label small">{m.relay_url()}</span>
					<code class="key__value">{relay.url}</code>
					<Button variant="text" size="s" onclick={() => relay && copy('url', relay.url)}><IconCopy size={16} />{copied === 'url' ? m.pj_copied() : m.pj_copy()}</Button>
				</div>
				<div class="key">
					<span class="key__label small">{m.relay_anon_key()}</span>
					<code class="key__value key__value--clip">{relay.anon_key}</code>
					<Button variant="text" size="s" onclick={() => relay && copy('anon', relay.anon_key)}><IconCopy size={16} />{copied === 'anon' ? m.pj_copied() : m.pj_copy()}</Button>
				</div>

				<h3 class="section-title">{m.relay_files()}</h3>
				{#if relay.buckets.length === 0}
					<p class="muted small">{m.relay_no_buckets()}</p>
				{:else}
					<ul class="list">
						{#each relay.buckets as bucket (bucket.name)}
							<li class="row">
								<span class="mono">{bucket.name}{bucket.public ? ` · ${m.relay_public()}` : ''}</span>
								<span class="muted small">{m.relay_bucket_rules({ read: RULES[bucket.read](), write: RULES[bucket.write](), size: bucket.max_size_mb })}</span>
							</li>
						{/each}
					</ul>
				{/if}

				<h3 class="section-title">{m.relay_live()}</h3>
				<p class="muted small">{relay.realtime.length ? relay.realtime.join(', ') : m.relay_no_live()}</p>

				<h3 class="section-title">{m.relay_functions()}</h3>
				{#if functions.length === 0}
					<p class="muted small">{m.relay_no_functions()}</p>
				{:else}
					<ul class="list">
						{#each functions as [name, fn] (name)}
							<li class="row">
								<span class="mono">{name}</span>
								<span class="muted small">
									{fn.auth === 'public' ? m.relay_function_public() : m.relay_function_signed_in()} · {m.relay_function_timeout({ seconds: fn.timeout })}{fn.schedule ? ` · ${m.relay_function_schedule({ schedule: fn.schedule })}` : ''}
								</span>
							</li>
						{/each}
					</ul>
					<p class="muted small">{m.relay_functions_version({ version: relay.functions?.version ?? 0 })}</p>
				{/if}

				<h3 class="section-title">{m.relay_sign_in()}</h3>
				<p class="muted small">{relay.auth.methods.map((method) => method.replace(/_/g, ' ')).join(', ')}</p>
				{#if relay.auth.missing_secrets.length}
					<p class="warn small">{m.relay_missing_secrets({ names: relay.auth.missing_secrets.join(', ') })}</p>
				{/if}
			</aside>

			<section class="console">
				{#if selected}
					<div class="structure">
						<p class="structure__title"><span class="mono">{selected.name}</span>
							<span class={selected.rls_enabled ? 'muted small' : 'warn small'}>{selected.rls_enabled ? m.relay_rls_on({ count: selected.policies.length }) : m.relay_rls_off()}</span>
						</p>
						<p class="columns small">
							{#each selected.columns as column (column.name)}
								<span class="column"><span class="mono">{column.name}</span> <span class="muted">{column.type}{column.nullable ? '?' : ''}</span></span>
							{/each}
						</p>
						{#each selected.policies as policy (policy.name)}
							<p class="policy small"><span class="mono">{policy.command}</span> {policy.name}{policy.using ? ` · using ${policy.using}` : ''}{policy.check ? ` · check ${policy.check}` : ''}</p>
						{/each}
					</div>
				{/if}
				<label class="sr-only" for="relay-sql">{m.pj_db_query()}</label>
				<textarea id="relay-sql" class="sql" bind:value={sql} onkeydown={keydown} spellcheck="false" placeholder="SELECT * FROM …" rows="6"></textarea>
				<div class="console__bar">
					<span class="muted small">{m.relay_console_hint()}</span>
					<Button variant="filled" size="s" disabled={running || !sql.trim()} onclick={run}>{running ? m.pj_db_running() : m.pj_db_run()}</Button>
				</div>
				{#if queryError}<p class="error" role="alert">{queryError}</p>{/if}
				{#if result}<QueryResultView {result} />{/if}
			</section>
		</div>
	{/if}
</div>

<style>
	.relay {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}
	.sources {
		display: inline-flex;
		align-self: flex-start;
		gap: 2px;
		padding: 3px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container-high);
	}
	.source {
		min-height: 36px;
		padding: 0 14px;
		border: none;
		border-radius: var(--md-sys-shape-corner-full);
		background: none;
		color: var(--md-sys-color-on-surface-variant);
		font: inherit;
		font-size: 0.875rem;
		font-weight: 600;
		cursor: pointer;
	}
	.source:hover:not(:disabled):not([aria-selected='true']) {
		color: var(--md-sys-color-on-surface);
	}
	.source[aria-selected='true'] {
		background: var(--md-sys-color-surface);
		color: var(--md-sys-color-on-surface);
	}
	.source:disabled {
		cursor: default;
		opacity: 0.5;
	}
	.source:focus-visible,
	.table:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.hint,
	.muted {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.small {
		font-size: 0.8125rem;
	}
	.warn {
		margin: 0;
		color: var(--md-sys-color-error);
	}
	.error {
		margin: 6px 0;
		color: var(--md-sys-color-error);
		overflow-wrap: anywhere;
	}
	.mono {
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.8125rem;
	}
	.layout {
		display: grid;
		grid-template-columns: 280px minmax(0, 1fr);
		gap: 20px;
		margin-top: 6px;
	}
	@media (max-width: 760px) {
		.layout {
			grid-template-columns: minmax(0, 1fr);
		}
	}
	.side {
		display: flex;
		flex-direction: column;
		gap: 6px;
		min-width: 0;
	}
	.section-title {
		margin: 12px 0 0;
		color: var(--md-sys-color-on-surface);
		font-size: 0.875rem;
		font-weight: 700;
	}
	.section-title:first-child {
		margin-top: 0;
	}
	.list {
		display: flex;
		flex-direction: column;
		gap: 2px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.row {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 4px 0;
	}
	.table {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 8px;
		width: 100%;
		min-height: 40px;
		padding: 0 10px;
		border: none;
		border-radius: 8px;
		background: none;
		color: var(--md-sys-color-on-surface);
		font: inherit;
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.8125rem;
		text-align: left;
		cursor: pointer;
	}
	.table:hover {
		background: var(--md-sys-color-surface-container);
	}
	.table[aria-current='true'] {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.table__name {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.key {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto;
		align-items: center;
		gap: 2px 6px;
	}
	.key__label {
		grid-column: 1 / -1;
		color: var(--md-sys-color-on-surface-variant);
	}
	.key__value {
		padding: 6px 8px;
		border-radius: 8px;
		background: var(--md-sys-color-surface-container-highest);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
		overflow-wrap: anywhere;
	}
	.key__value--clip {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.console {
		display: flex;
		flex-direction: column;
		gap: 8px;
		min-width: 0;
	}
	.structure {
		display: flex;
		flex-direction: column;
		gap: 6px;
		padding: 12px 14px;
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-surface-container-low);
	}
	.structure__title {
		display: flex;
		flex-wrap: wrap;
		align-items: baseline;
		gap: 4px 10px;
		margin: 0;
		font-weight: 700;
	}
	.columns {
		display: flex;
		flex-wrap: wrap;
		gap: 4px 14px;
		margin: 0;
	}
	.policy {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		overflow-wrap: anywhere;
	}
	.sql {
		box-sizing: border-box;
		width: 100%;
		padding: 12px 14px;
		border: 1.5px solid var(--md-sys-color-outline);
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-surface);
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.8125rem;
		resize: vertical;
	}
	.sql:focus {
		outline: none;
		border-color: var(--md-sys-color-primary);
	}
	.console__bar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
	}
</style>
