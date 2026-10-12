<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { untrack } from 'svelte';
	// Every app the crew can work in, grouped by kind and searchable; each opens its own setup page.
	import { CATEGORY_ORDER, CONNECTORS, categoryLabel, searchConnectors, type Connector } from '$lib/connectors';
	import { timeAgo } from '$lib/i18n';
	import { crewName } from '$lib/crewNames';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	// Seeded once from ?q=; typing takes over after that.
	let query = $state(untrack(() => data.query));
	const found = $derived(searchConnectors(CONNECTORS, query));
	const groups = $derived(
		CATEGORY_ORDER.map((category) => ({ category, connectors: found.filter((c) => c.category === category) })).filter((g) => g.connectors.length > 0),
	);

	/** What the row says about this person's connection. */
	function status(connector: Connector): { connected: boolean; text: string } {
		const overview = data.overview;
		if (connector.soon) return { connected: false, text: m.connectors_soon() };
		if (connector.id === 'google' && overview?.google.email) return { connected: true, text: overview.google.email };
		if (connector.id === 'hevy' && overview?.hevy) {
			return { connected: true, text: overview.hevy.account_name ?? m.connectors_connected_since({ when: timeAgo(overview.hevy.connected_at) }) };
		}
		return { connected: false, text: m.conn_not_connected() };
	}
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
							{#if connector.soon}
								<div class="row row--soon" aria-disabled="true">
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
	</div>
</div>

{#snippet body(connector: Connector, state: { connected: boolean; text: string })}
	<span class="row__icon" class:row__icon--on={state.connected}>
		<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d={connector.icon} /></svg>
	</span>
	<span class="row__text">
		<span class="row__name">{connector.name}</span>
		<span class="row__desc">{connector.description()}</span>
		<span class="row__meta">
			<span class:row__state--on={state.connected}>{state.text}</span>
			· {m.connectors_used_by({ agent: crewName(connector.agent) })}
		</span>
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
	.row__chevron {
		flex: none;
		color: var(--md-sys-color-on-surface-variant);
	}
</style>
