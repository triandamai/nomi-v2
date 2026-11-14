<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { untrack } from 'svelte';
	// Every app the crew can work in, grouped by kind and searchable; each opens its own setup page.
	import { CATEGORY_ORDER, CONNECTORS, categoryLabel, connectorState, searchConnectors, withCatalog, type Connector, type ConnectorState } from '$lib/connectors';
	import { crewName } from '$lib/crewNames';
	import { enhance } from '$app/forms';
	import Button from '$lib/components/m3/Button.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	// Seeded once from ?q=; typing takes over after that. The server's catalog adds the apps
	// coming next (and GitHub, connected from a project's Export).
	let query = $state(untrack(() => data.query));
	const all = $derived(withCatalog(CONNECTORS, data.catalog));
	const found = $derived(searchConnectors(all, query));
	const groups = $derived(
		CATEGORY_ORDER.map((category) => ({ category, connectors: found.filter((c) => c.category === category) })).filter((g) => g.connectors.length > 0),
	);

	/** What the row says about this person's connection. */
	const status = (connector: Connector): ConnectorState => connectorState(connector, data.overview, data.github);

	let mcpName = $state(untrack(() => (form && 'name' in form ? String(form.name ?? '') : '')));
	let mcpUrl = $state(untrack(() => (form && 'url' in form ? String(form.url ?? '') : '')));
	let addingMcp = $state(false);
</script>

<div class="connections">
	<div class="connections__inner">
		<header>
			<h1 class="md-display-small title">{m.conn_title()}</h1>
			<p class="md-body-large lede">{m.connectors_lede()}</p>
		</header>

		{#if !data.overview}
			<p class="notice" role="alert">{m.conn_load_failed()}</p>
		{/if}

		<label class="search">
			<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true"><circle cx="11" cy="11" r="7" /><path d="m20 20-3.5-3.5" /></svg>
			<span class="sr-only">{m.connectors_search()}</span>
			<input type="search" placeholder={m.connectors_search()} bind:value={query} autocomplete="off" />
		</label>

		{#if groups.length === 0}
			<p class="empty">{m.list_no_match({ query })}</p>
		{/if}

		{#each groups as group (group.category)}
			<section class="group" aria-labelledby="group-{group.category}">
				<h2 class="group__title" id="group-{group.category}">{categoryLabel(group.category)}</h2>
				<ul class="rows">
					{#each group.connectors as connector (connector.id)}
						{@const state = status(connector)}
						<li>
							{#if connector.soon || connector.elsewhere}
								<div class="row" class:row--soon={connector.soon} aria-disabled={connector.soon ? 'true' : undefined}>
									{@render body(connector, state)}
								</div>
							{:else}
								<a class="row" href="/connections/{connector.id}">
									{@render body(connector, state)}
									<svg class="row__chevron" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="m9 6 6 6-6 6" /></svg>
								</a>
							{/if}
						</li>
					{/each}
				</ul>
			</section>
		{/each}

		<section class="group mcp" aria-labelledby="mcp-title">
			<h2 class="group__title" id="mcp-title">{m.mcp_title()}</h2>
			<p class="mcp__lede">{m.mcp_lede()}</p>
			{#if !data.mcp.callable}
				<p class="mcp__lede">{m.mcp_not_callable()}</p>
			{/if}
			{#if data.mcp.servers.length === 0}
				<p class="empty">{m.mcp_empty()}</p>
			{:else}
				<ul class="rows">
					{#each data.mcp.servers as server (server.id)}
						<li class="row">
							<span class="row__text">
								<span class="row__name">{server.name}</span>
								<span class="row__meta">{server.url}</span>
							</span>
							<form method="POST" action="?/removeMcp" use:enhance>
								<input type="hidden" name="id" value={server.id} />
								<Button variant="text" size="xs" type="submit" aria-label={m.mcp_remove({ name: server.name })}>{m.mcp_remove_short()}</Button>
							</form>
						</li>
					{/each}
				</ul>
			{/if}
			<form
				class="mcp__form"
				method="POST"
				action="?/addMcp"
				use:enhance={() => {
					addingMcp = true;
					return async ({ result, update }) => {
						await update({ reset: false });
						addingMcp = false;
						if (result.type === 'success') {
							mcpName = '';
							mcpUrl = '';
						}
					};
				}}
			>
				<TextField label={m.mcp_name()} name="name" bind:value={mcpName} required maxlength={80} />
				<TextField label={m.mcp_url()} name="url" type="url" bind:value={mcpUrl} required placeholder="https://" />
				{#if form && 'mcpError' in form && form.mcpError}
					<p class="notice" role="alert">{form.mcpError}</p>
				{:else if form && 'mcpAdded' in form}
					<p class="mcp__lede" role="status">{m.mcp_added()}</p>
				{/if}
				<Button variant="tonal" type="submit" disabled={addingMcp}>{m.mcp_add()}</Button>
			</form>
		</section>
	</div>
</div>

{#snippet body(connector: Connector, state: ConnectorState)}
	<span class="row__icon" class:row__icon--on={state.connected}>
		<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d={connector.icon} /></svg>
	</span>
	<span class="row__text">
		<span class="row__name">{connector.name}</span>
		<span class="row__desc">{connector.description()}</span>
		<span class="row__meta">
			<span class:row__state--on={state.connected} class:row__state--needs={state.needsYou}>{state.text}</span>
			· {state.usedBy.length
				? m.connectors_used_by({ agent: state.usedBy.map(crewName).join(', ') })
				: m.connectors_used_by_you()}
		</span>
		{#if state.access.length}
			<span class="row__meta">{state.access.join(' · ')}</span>
		{/if}
		{#if state.needsYou}
			<span class="row__reconnect">{m.connectors_reconnect()}</span>
		{/if}
	</span>
{/snippet}

<style>
	.connections {
		height: 100%;
		overflow-y: auto;
		padding: 24px clamp(14px, 4vw, 44px) 44px;
		box-sizing: border-box;
	}
	.connections__inner {
		display: flex;
		flex-direction: column;
		gap: 16px;
		max-width: 720px;
		margin: 0 auto;
	}
	.title {
		margin: 0;
		color: var(--md-sys-color-on-surface);
	}
	.lede {
		margin: 8px 0 0;
		max-width: 60ch;
		color: var(--md-sys-color-on-surface-variant);
	}
	.notice {
		margin: 0;
		color: var(--md-sys-color-error);
	}
	.search {
		display: flex;
		align-items: center;
		gap: 10px;
		min-height: 52px;
		padding: 0 14px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface-variant);
	}
	.search:focus-within {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.search input {
		flex: 1;
		min-width: 0;
		border: 0;
		background: transparent;
		color: var(--md-sys-color-on-surface);
		font: inherit;
		outline: none;
	}
	.sr-only {
		position: absolute;
		width: 1px;
		height: 1px;
		overflow: hidden;
		clip: rect(0 0 0 0);
		white-space: nowrap;
	}
	.empty {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.group {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.group__title {
		margin: 0 4px;
		font-size: 0.9375rem;
		font-weight: 650;
		color: var(--md-sys-color-on-surface-variant);
	}
	.rows {
		display: flex;
		flex-direction: column;
		gap: 2px;
		margin: 0;
		padding: 0;
		overflow: hidden;
		border-radius: var(--md-sys-shape-corner-extra-large);
		list-style: none;
	}
	.row {
		display: flex;
		align-items: center;
		gap: 12px;
		min-height: 72px;
		padding: 12px 14px;
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
		text-decoration: none;
		transition: background-color 150ms ease-out;
	}
	a.row:hover {
		background: var(--md-sys-color-surface-container-low);
	}
	a.row:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: -2px;
	}
	.row--soon {
		opacity: 0.62;
	}
	.row__icon {
		display: grid;
		flex: none;
		place-items: center;
		width: 44px;
		height: 44px;
		border-radius: 14px;
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface-variant);
	}
	.row__icon--on {
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
	}
	.row__text {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.row__name {
		font-weight: 650;
	}
	.row__desc {
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.row__meta {
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
		overflow-wrap: anywhere;
	}
	.row__state--on {
		color: var(--md-sys-color-primary);
		font-weight: 600;
	}
	.row__state--needs {
		color: var(--md-sys-color-error);
		font-weight: 600;
	}
	.row__reconnect {
		align-self: flex-start;
		margin-top: 4px;
		padding: 2px 10px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-error-container);
		color: var(--md-sys-color-on-error-container);
		font-size: 0.8125rem;
		font-weight: 600;
	}
	.mcp__lede {
		margin: 0 4px;
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.mcp__form {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 14px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
	}
	.row__chevron {
		flex: none;
		color: var(--md-sys-color-on-surface-variant);
	}
</style>
