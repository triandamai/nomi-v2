<script lang="ts">
	// The project's databases on Shipyard: the development one and the published app's. Its
	// tables, a SQL console, and the login for the admin's own database client.
	import { m } from '$lib/paraglide/messages';
	import Button from '$lib/components/m3/Button.svelte';
	import IconCopy from '$lib/components/icons/IconCopy.svelte';
	import QueryResultView from '$lib/components/project/QueryResultView.svelte';
	import { projectApi } from '$lib/projectApi';
	import type { DatabaseSource, DatabaseTable, QueryResult } from '$lib/types';

	let { projectId, published }: { projectId: string; published: boolean } = $props();

	let source = $state<DatabaseSource>('development');
	let tables = $state<DatabaseTable[] | null>(null);
	let tablesError = $state<string | null>(null);
	let sql = $state('');
	let running = $state(false);
	let result = $state<QueryResult | null>(null);
	let queryError = $state<string | null>(null);
	let login = $state<string | null>(null);
	let loginError = $state<string | null>(null);
	let copied = $state(false);

	async function loadTables() {
		tables = null;
		tablesError = null;
		try {
			tables = await projectApi<DatabaseTable[]>(projectId, `database/tables?source=${source}`);
		} catch (e) {
			tablesError = (e as Error).message;
		}
	}

	$effect(() => {
		void source;
		login = null;
		result = null;
		void loadTables();
	});

	async function run() {
		if (!sql.trim()) return;
		running = true;
		queryError = null;
		try {
			result = await projectApi<QueryResult>(projectId, 'database/query', { method: 'POST', body: { sql, source } });
			if (result.rows_affected !== null) void loadTables();
		} catch (e) {
			queryError = (e as Error).message;
			result = null;
		} finally {
			running = false;
		}
	}

	function browse(table: DatabaseTable) {
		sql = `SELECT * FROM "${table.schema}"."${table.name}" LIMIT 100`;
		void run();
	}

	async function showLogin() {
		loginError = null;
		try {
			login = (await projectApi<{ url: string }>(projectId, `database/credentials?source=${source}`, { method: 'POST' })).url;
		} catch (e) {
			loginError = (e as Error).message;
		}
	}

	async function copyLogin() {
		if (!login) return;
		await navigator.clipboard.writeText(login).catch(() => {});
		copied = true;
		setTimeout(() => (copied = false), 1500);
	}

	function keydown(event: KeyboardEvent) {
		if ((event.metaKey || event.ctrlKey) && event.key === 'Enter') {
			event.preventDefault();
			void run();
		}
	}
</script>

<div class="db">
	<div class="sources" role="tablist" aria-label={m.pj_db_source()}>
		<button type="button" role="tab" class="source" aria-selected={source === 'development'} onclick={() => (source = 'development')}>{m.pj_db_development()}</button>
		<button type="button" role="tab" class="source" aria-selected={source === 'production'} disabled={!published} onclick={() => (source = 'production')}>{m.pj_db_production()}</button>
	</div>
	<p class="hint">{source === 'development' ? m.pj_db_development_hint() : m.pj_db_production_hint()}</p>

	<div class="layout">
		<aside class="tables">
			<h3 class="section-title">{m.pj_db_tables()}</h3>
			{#if tablesError}
				<p class="error">{tablesError}</p>
			{:else if !tables}
				<p class="muted">{m.pj_loading()}</p>
			{:else if tables.length === 0}
				<p class="muted">{m.pj_db_no_tables()}</p>
			{:else}
				<ul class="table-list">
					{#each tables as table (table.schema + '.' + table.name)}
						<li>
							<button type="button" class="table" onclick={() => browse(table)}>
								<span class="table__name">{table.schema === 'public' ? table.name : `${table.schema}.${table.name}`}</span>
								{#if table.rows_estimate >= 0}<span class="table__rows">{table.rows_estimate}</span>{/if}
							</button>
						</li>
					{/each}
				</ul>
			{/if}

			<h3 class="section-title">{m.pj_db_login()}</h3>
			{#if login}
				<code class="login">{login}</code>
				<Button variant="tonal" size="s" onclick={copyLogin}><IconCopy size={16} />{copied ? m.pj_copied() : m.pj_copy()}</Button>
			{:else}
				<p class="muted">{m.pj_db_login_hint()}</p>
				<Button variant="outlined" size="s" onclick={showLogin}>{m.pj_db_show_login()}</Button>
			{/if}
			{#if loginError}<p class="error">{loginError}</p>{/if}
		</aside>

		<section class="console">
			<label class="sr-only" for="sql">{m.pj_db_query()}</label>
			<textarea id="sql" class="sql" bind:value={sql} onkeydown={keydown} spellcheck="false" placeholder="SELECT * FROM …" rows="6"></textarea>
			<div class="console__bar">
				<span class="muted small">{m.pj_db_run_hint()}</span>
				<Button variant="filled" size="s" disabled={running || !sql.trim()} onclick={run}>{running ? m.pj_db_running() : m.pj_db_run()}</Button>
			</div>
			{#if queryError}<p class="error" role="alert">{queryError}</p>{/if}
			{#if result}<QueryResultView {result} />{/if}
		</section>
	</div>
</div>

<style>
	.db {
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
	.error {
		margin: 6px 0;
		color: var(--md-sys-color-error);
		overflow-wrap: anywhere;
	}
	.layout {
		display: grid;
		grid-template-columns: 240px minmax(0, 1fr);
		gap: 20px;
		margin-top: 6px;
	}
	@media (max-width: 760px) {
		.layout {
			grid-template-columns: minmax(0, 1fr);
		}
	}
	.tables {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 8px;
		min-width: 0;
	}
	.section-title {
		margin: 8px 0 0;
		color: var(--md-sys-color-on-surface);
		font-size: 0.875rem;
		font-weight: 700;
	}
	.table-list {
		width: 100%;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.table {
		display: flex;
		justify-content: space-between;
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
		align-items: center;
		cursor: pointer;
	}
	.table:hover {
		background: var(--md-sys-color-surface-container);
	}
	.table__name {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.table__rows {
		color: var(--md-sys-color-on-surface-variant);
	}
	.login {
		display: block;
		width: 100%;
		box-sizing: border-box;
		padding: 8px 10px;
		border-radius: 8px;
		background: var(--md-sys-color-surface-container-highest);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
		overflow-wrap: anywhere;
	}
	.console {
		display: flex;
		flex-direction: column;
		gap: 8px;
		min-width: 0;
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
