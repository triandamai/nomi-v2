<script lang="ts">
	// The providers someone (or, on the admin page, everyone) has connected, and the popular ones
	// still to connect. Connecting and disconnecting go back to the page through callbacks; a
	// disconnect is confirmed here first, and refused while models still run on the connection.
	import { m } from '$lib/paraglide/messages';
	import Button from '$lib/components/m3/Button.svelte';
	import Dialog from '$lib/components/m3/Dialog.svelte';
	import IconChevronRight from '$lib/components/icons/IconChevronRight.svelte';
	import IconPlus from '$lib/components/icons/IconPlus.svelte';
	import ProviderLogo from './ProviderLogo.svelte';
	import { authLabel, connectionName, type ProviderConnection, type ProviderDefinition } from '$lib/providers';

	let {
		connections,
		providers,
		onconnect,
		ondisconnect,
		onbrowse,
	}: {
		connections: ProviderConnection[];
		providers: ProviderDefinition[];
		onconnect: (provider: ProviderDefinition) => void;
		ondisconnect: (connection: ProviderConnection) => void;
		onbrowse: () => void;
	} = $props();

	const connectedIds = $derived(new Set(connections.map((c) => c.provider_id)));
	const popular = $derived(providers.filter((p) => p.category === 'popular' && !connectedIds.has(p.id)));
	const hasLogo = (providerId: string) => providers.find((p) => p.id === providerId)?.has_logo ?? false;

	let confirmOpen = $state(false);
	let pending = $state<ProviderConnection | null>(null);
	const pendingName = $derived(pending ? connectionName(pending, providers) : '');
	const blocked = $derived((pending?.models_in_use.length ?? 0) > 0);

	function askDisconnect(connection: ProviderConnection) {
		pending = connection;
		confirmOpen = true;
	}

	function confirmDisconnect() {
		if (!pending || blocked) return;
		const connection = pending;
		confirmOpen = false;
		ondisconnect(connection);
	}
</script>

<div class="panel">
	<section class="group" aria-labelledby="prov-connected">
		<h2 class="group__title" id="prov-connected">{m.prov_connected_title()}</h2>
		{#if connections.length === 0}
			<p class="empty">{m.prov_connected_none()}</p>
		{:else}
			<ul class="rows">
				{#each connections as connection (connection.id)}
					{@const name = connectionName(connection, providers)}
					<li class="row" data-testid="connection">
						<span class="row__logo"><ProviderLogo id={connection.provider_id} hasLogo={hasLogo(connection.provider_id)} size={22} /></span>
						<span class="row__text">
							<span class="row__head">
								<span class="row__name">{name}</span>
								<span class="badge">{authLabel(connection.auth_type)}</span>
							</span>
							{#if connection.base_url || connection.api_key_masked}
								<span class="row__meta">{connection.auth_type === 'custom_endpoint' ? (connection.base_url ?? connection.api_key_masked) : (connection.api_key_masked ?? connection.base_url)}</span>
							{/if}
						</span>
						<Button variant="text" size="s" class="row__action" aria-label={m.prov_disconnect_named({ name })} onclick={() => askDisconnect(connection)}>
							{m.conn_disconnect()}
						</Button>
					</li>
				{/each}
			</ul>
		{/if}
	</section>

	<section class="group" aria-labelledby="prov-popular" data-testid="popular">
		<h2 class="group__title" id="prov-popular">{m.prov_popular_title()}</h2>
		{#if popular.length > 0}
			<ul class="rows">
				{#each popular as provider (provider.id)}
					<li class="row">
						<span class="row__logo"><ProviderLogo id={provider.id} hasLogo={provider.has_logo} size={22} /></span>
						<span class="row__text">
							<span class="row__head">
								<span class="row__name">{provider.name}</span>
								{#if provider.recommended}<span class="badge badge--accent">{m.prov_recommended()}</span>{/if}
							</span>
							<span class="row__meta row__meta--clamp">{provider.description}</span>
						</span>
						<Button variant="tonal" size="s" class="row__action" aria-label={m.prov_connect_named({ name: provider.name })} onclick={() => onconnect(provider)}>
							<IconPlus size={18} />
							{m.prov_connect()}
						</Button>
					</li>
				{/each}
			</ul>
		{/if}
		<div class="browse">
			<Button variant="text" onclick={onbrowse}>{m.prov_browse()} <IconChevronRight size={18} /></Button>
		</div>
	</section>
</div>

<Dialog bind:open={confirmOpen} headline={m.prov_disconnect_title({ name: pendingName })} class="prov-confirm">
	<p class="dialog-text">
		{#if blocked && pending}
			{m.prov_in_use({ models: pending.models_in_use.join(', ') })}
		{:else}
			{m.prov_disconnect_body({ name: pendingName })}
		{/if}
	</p>
	{#snippet actions()}
		<Button variant="text" onclick={() => (confirmOpen = false)}>{m.common_cancel()}</Button>
		<Button variant="filled" class="danger" disabled={blocked} onclick={confirmDisconnect}>{m.conn_disconnect()}</Button>
	{/snippet}
</Dialog>

<style>
	.panel {
		display: flex;
		flex-direction: column;
		gap: 28px;
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
	.empty {
		margin: 0;
		padding: 18px 16px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface-variant);
	}
	.rows {
		margin: 0;
		padding: 0;
		overflow: hidden;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
		list-style: none;
	}
	.row {
		display: flex;
		align-items: center;
		gap: 12px;
		min-height: 64px;
		padding: 10px 10px 10px 14px;
		color: var(--md-sys-color-on-surface);
	}
	.row + .row {
		border-top: 1px solid var(--md-sys-color-outline-variant);
	}
	.row__logo {
		display: grid;
		flex: none;
		place-items: center;
		width: 40px;
		height: 40px;
		border-radius: 12px;
		background: var(--md-sys-color-surface-container-low);
		color: var(--md-sys-color-on-surface);
	}
	.row__text {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.row__head {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 6px 8px;
	}
	.row__name {
		font-weight: 650;
		overflow-wrap: anywhere;
	}
	.row__meta {
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
		overflow-wrap: anywhere;
	}
	.row__meta--clamp {
		display: -webkit-box;
		overflow: hidden;
		-webkit-box-orient: vertical;
		-webkit-line-clamp: 2;
		line-clamp: 2;
	}
	.row :global(.row__action) {
		flex: none;
	}
	.badge {
		padding: 2px 8px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.75rem;
		font-weight: 600;
		white-space: nowrap;
	}
	.badge--accent {
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
	}
	.browse {
		display: flex;
	}
	.dialog-text {
		margin: 0;
	}
	:global(dialog.prov-confirm .danger:not(:disabled)) {
		background: var(--md-sys-color-error);
		color: var(--md-sys-color-on-error);
	}
	/* Phones: a 44px target for every action in a row. */
	@media (max-width: 640px) {
		.row :global(.row__action),
		.browse :global(.m3-button) {
			min-height: 44px;
		}
	}
</style>
