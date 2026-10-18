<script lang="ts">
	// Connecting a provider: a searchable list (Popular, then Other), then that provider's form
	// (an API key, or a custom endpoint's name, base URL, key and headers). The page does the
	// saving through `onsubmit` and hands back `busy` and the server's `error`.
	import { tick, untrack } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import Button from '$lib/components/m3/Button.svelte';
	import Dialog from '$lib/components/m3/Dialog.svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import IconArrowBack from '$lib/components/icons/IconArrowBack.svelte';
	import IconClose from '$lib/components/icons/IconClose.svelte';
	import IconOpenInNew from '$lib/components/icons/IconOpenInNew.svelte';
	import IconPlus from '$lib/components/icons/IconPlus.svelte';
	import IconSearch from '$lib/components/icons/IconSearch.svelte';
	import ProviderLogo from './ProviderLogo.svelte';
	import {
		CUSTOM_PROVIDER_ID,
		groupProviders,
		searchProviders,
		stepHighlight,
		type ConnectInput,
		type ProviderDefinition,
	} from '$lib/providers';

	let {
		open = $bindable(false),
		providers,
		connectedIds,
		initialProviderId = null,
		error = null,
		busy = false,
		onsubmit,
	}: {
		open?: boolean;
		providers: ProviderDefinition[];
		/** Providers already connected: marked, and not offered again (custom endpoints can repeat). */
		connectedIds: string[];
		/** Opens straight on this provider's form. */
		initialProviderId?: string | null;
		error?: string | null;
		busy?: boolean;
		onsubmit: (input: ConnectInput) => void;
	} = $props();

	const uid = $props.id();
	const optionId = (provider: ProviderDefinition) => `${uid}-opt-${provider.id}`;

	let query = $state('');
	let highlighted = $state(0);
	let selected = $state<ProviderDefinition | null>(null);

	const isConnected = (provider: ProviderDefinition) => provider.id !== CUSTOM_PROVIDER_ID && connectedIds.includes(provider.id);
	const groups = $derived(groupProviders(searchProviders(providers, query)));
	// Arrow keys walk the providers that can be picked, in the order they're shown.
	const choices = $derived(groups.flatMap((g) => g.providers).filter((p) => !isConnected(p)));
	const activeId = $derived(choices[highlighted] ? optionId(choices[highlighted]) : undefined);

	// The form for the selected provider.
	let apiKey = $state('');
	let label = $state('');
	let baseUrl = $state('');
	let headers = $state<{ key: number; name: string; value: string }[]>([]);
	let nextHeaderKey = 0;
	// The page's error belongs to the last attempt: going back to the list leaves it behind.
	let errorStale = $state(false);

	const custom = $derived(selected?.auth_type === 'custom_endpoint');
	const oauth = $derived(selected?.auth_type === 'oauth');
	const ready = $derived(custom ? !!label.trim() && !!baseUrl.trim() : !oauth && !!apiKey.trim());

	let searchEl: HTMLInputElement | undefined = $state();
	let detailEl: HTMLElement | undefined = $state();

	// Every opening starts afresh, on the given provider's form or on the list.
	let wasOpen = false;
	$effect(() => {
		const opening = open && !wasOpen;
		wasOpen = open;
		if (!opening) return;
		untrack(() => {
			query = '';
			highlighted = 0;
			const initial = initialProviderId ? providers.find((p) => p.id === initialProviderId) : undefined;
			if (initial && !isConnected(initial)) select(initial);
			else showList();
			errorStale = false;
		});
	});

	async function select(provider: ProviderDefinition) {
		if (isConnected(provider)) return;
		selected = provider;
		apiKey = '';
		label = provider.auth_type === 'custom_endpoint' && provider.id !== CUSTOM_PROVIDER_ID ? provider.name : '';
		baseUrl = provider.default_base_url ?? '';
		headers = [];
		await tick();
		detailEl?.querySelector('input')?.focus();
	}

	async function showList() {
		selected = null;
		errorStale = true;
		await tick();
		searchEl?.focus();
	}

	function onSearchKey(event: KeyboardEvent) {
		if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
			event.preventDefault();
			highlighted = stepHighlight(highlighted, choices.length, event.key);
			if (activeId) document.getElementById(activeId)?.scrollIntoView?.({ block: 'nearest' });
		} else if (event.key === 'Enter') {
			event.preventDefault();
			const pick = choices[highlighted];
			if (pick) select(pick);
		}
	}

	function onKey(event: KeyboardEvent) {
		// Handled here rather than by the native dialog so the person's place resets the same way.
		if (event.key === 'Escape') {
			event.preventDefault();
			open = false;
		}
	}

	function addHeader() {
		headers = [...headers, { key: nextHeaderKey++, name: '', value: '' }];
	}

	function submit(event: SubmitEvent) {
		event.preventDefault();
		if (!selected || !ready || busy) return;
		errorStale = false;
		const input: ConnectInput = { provider_id: selected.id };
		if (custom) {
			input.label = label.trim();
			input.base_url = baseUrl.trim();
			if (apiKey.trim()) input.api_key = apiKey.trim();
			const named = headers.filter((h) => h.name.trim()).map((h) => ({ name: h.name.trim(), value: h.value }));
			if (named.length > 0) input.headers = named;
		} else {
			input.api_key = apiKey.trim();
		}
		onsubmit(input);
	}
</script>

<Dialog bind:open class="provider-dialog">
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="connect" onkeydown={onKey}>
		{#if !selected}
			<div class="head">
				<h2 class="title">{m.prov_dialog_title()}</h2>
				<IconButton aria-label={m.common_close()} onclick={() => (open = false)}><IconClose size={20} /></IconButton>
			</div>
			<label class="search">
				<IconSearch size={18} />
				<input
					bind:this={searchEl}
					type="text"
					role="combobox"
					aria-label={m.prov_search()}
					aria-expanded="true"
					aria-controls="{uid}-list"
					aria-autocomplete="list"
					aria-activedescendant={activeId}
					placeholder={m.prov_search()}
					autocomplete="off"
					spellcheck="false"
					bind:value={query}
					oninput={() => (highlighted = 0)}
					onkeydown={onSearchKey}
				/>
			</label>
			<div class="list" id="{uid}-list" role="listbox" aria-label={m.prov_dialog_title()}>
				{#each groups as group (group.category)}
					{@const groupLabel = group.category === 'popular' ? m.prov_group_popular() : m.prov_group_other()}
					<div class="group" role="group" aria-label={groupLabel}>
						<p class="group__title" aria-hidden="true">{groupLabel}</p>
						{#each group.providers as provider (provider.id)}
							{@const connected = isConnected(provider)}
							{@const active = activeId === optionId(provider)}
							<!-- Focus stays in the search field (aria-activedescendant); the mouse picks here. -->
							<div
								id={optionId(provider)}
								role="option"
								tabindex="-1"
								aria-selected={active}
								aria-disabled={connected}
								class="option"
								class:option--active={active}
								class:option--off={connected}
								onmousedown={(e) => e.preventDefault()}
								onclick={() => select(provider)}
								onkeydown={() => {}}
								onmouseenter={() => {
									const at = choices.indexOf(provider);
									if (at >= 0) highlighted = at;
								}}
							>
								<span class="option__logo"><ProviderLogo id={provider.id} hasLogo={provider.has_logo} size={20} /></span>
								<span class="option__text">
									<span class="option__name" data-name>{provider.name}</span>
									<span class="option__desc">{provider.description}</span>
								</span>
								{#if connected}
									<span class="badge badge--on">{m.prov_connected_badge()}</span>
								{:else if provider.category === 'custom'}
									<span class="badge">{m.prov_custom_badge()}</span>
								{:else if provider.recommended}
									<span class="badge badge--accent">{m.prov_recommended()}</span>
								{/if}
							</div>
						{/each}
					</div>
				{/each}
				{#if providers.length === 0}
					<p class="empty" role="alert">{m.prov_load_failed()}</p>
				{:else if groups.length === 0}
					<p class="empty">{m.list_no_match({ query: query.trim() })}</p>
				{/if}
			</div>
		{:else}
			{@const name = selected.name}
			<div class="head">
				<IconButton aria-label={m.prov_back()} onclick={showList}><IconArrowBack size={20} /></IconButton>
				<IconButton aria-label={m.common_close()} onclick={() => (open = false)}><IconClose size={20} /></IconButton>
			</div>
			<form class="detail" bind:this={detailEl} onsubmit={submit} novalidate>
				<div class="detail__title">
					<span class="option__logo option__logo--big"><ProviderLogo id={selected.id} hasLogo={selected.has_logo} size={26} /></span>
					<h2 class="title">{m.prov_connect_named({ name })}</h2>
				</div>
				<p class="detail__lede">
					{#if custom}
						{m.prov_custom_instructions()}
					{:else if oauth}
						{m.prov_oauth_unsupported({ name })}
					{:else}
						{m.prov_instructions({ name })}
					{/if}
					{#if selected.doc_url}
						<a class="docs" href={selected.doc_url} target="_blank" rel="noopener noreferrer">
							{m.prov_docs({ name })}<IconOpenInNew size={14} />
						</a>
					{/if}
				</p>

				{#if custom}
					<TextField label={m.prov_custom_name()} bind:value={label} autocomplete="off" required />
					<TextField label={m.prov_base_url()} bind:value={baseUrl} type="url" inputmode="url" placeholder="https://api.example.com/v1" autocomplete="off" spellcheck="false" required />
					<TextField label={m.prov_api_key_optional()} bind:value={apiKey} type="password" autocomplete="off" />
					<fieldset class="headers">
						<legend class="headers__legend">{m.prov_headers()}</legend>
						<p class="headers__hint">{m.prov_headers_hint()}</p>
						{#each headers as header (header.key)}
							<div class="header-row">
								<TextField label={m.prov_header_name()} bind:value={header.name} data-header="name" autocomplete="off" spellcheck="false" />
								<TextField label={m.prov_header_value()} bind:value={header.value} data-header="value" autocomplete="off" spellcheck="false" />
								<IconButton
									aria-label={m.prov_header_remove()}
									class="header-row__remove"
									onclick={() => (headers = headers.filter((h) => h.key !== header.key))}
								>
									<IconClose size={18} />
								</IconButton>
							</div>
						{/each}
						<div><Button type="button" variant="text" size="s" onclick={addHeader}><IconPlus size={18} /> {m.prov_header_add()}</Button></div>
					</fieldset>
				{:else if !oauth}
					<TextField label={m.prov_key_label({ name })} bind:value={apiKey} type="password" autocomplete="off" spellcheck="false" required />
				{/if}

				{#if error && !errorStale}
					<p class="error" role="alert">{error}</p>
				{/if}

				{#if !oauth}
					<div class="actions">
						<Button type="submit" variant="filled" disabled={!ready || busy} aria-busy={busy}>
							{busy ? m.prov_connecting() : m.prov_continue()}
						</Button>
					</div>
				{/if}
			</form>
		{/if}
	</div>
</Dialog>

<style>
	:global(dialog.m3-dialog.provider-dialog) {
		width: min(600px, calc(100vw - 32px));
		max-width: none;
		min-width: 0;
		background: var(--md-sys-color-surface-container-high);
	}
	:global(dialog.m3-dialog.provider-dialog .m3-dialog__content) {
		padding: 12px 12px 16px;
	}
	:global(dialog.m3-dialog.provider-dialog .m3-dialog__body) {
		color: var(--md-sys-color-on-surface);
	}
	.connect {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
	.head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		padding-left: 8px;
	}
	.title {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.25rem;
		font-weight: 700;
		color: var(--md-sys-color-on-surface);
		overflow-wrap: anywhere;
	}
	.search {
		display: flex;
		align-items: center;
		gap: 10px;
		min-height: 48px;
		margin: 0 4px;
		padding: 0 14px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface-variant);
	}
	.search:focus-within {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 0;
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
	.list {
		display: flex;
		flex-direction: column;
		gap: 12px;
		max-height: min(56vh, 520px);
		overflow-y: auto;
		overscroll-behavior: contain;
		padding: 0 4px 4px;
	}
	.group {
		display: flex;
		flex-direction: column;
	}
	.group__title {
		margin: 0 0 4px;
		padding: 0 10px;
		font-size: 0.8125rem;
		font-weight: 650;
		color: var(--md-sys-color-on-surface-variant);
	}
	.option {
		display: flex;
		align-items: center;
		gap: 12px;
		min-height: 52px;
		padding: 6px 10px;
		border-radius: var(--md-sys-shape-corner-large);
		color: var(--md-sys-color-on-surface);
		cursor: pointer;
	}
	.option--active {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.option--off {
		cursor: default;
		opacity: 0.6;
	}
	.option__logo {
		display: grid;
		flex: none;
		place-items: center;
		width: 36px;
		height: 36px;
		border-radius: 10px;
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
	}
	.option__logo--big {
		width: 48px;
		height: 48px;
		border-radius: 14px;
	}
	.option__text {
		display: flex;
		flex: 1;
		flex-wrap: wrap;
		align-items: baseline;
		gap: 0 8px;
		min-width: 0;
	}
	.option__name {
		font-weight: 600;
	}
	.option__desc {
		flex: 1 1 12ch;
		min-width: 0;
		overflow: hidden;
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.option--active .option__desc {
		color: inherit;
		opacity: 0.8;
	}
	.badge {
		flex: none;
		padding: 2px 8px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container-highest);
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.75rem;
		font-weight: 600;
	}
	.badge--accent {
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
	}
	.badge--on {
		background: var(--md-sys-color-tertiary-container);
		color: var(--md-sys-color-on-tertiary-container);
	}
	.empty {
		margin: 0;
		padding: 12px 10px;
		color: var(--md-sys-color-on-surface-variant);
	}
	.detail {
		display: flex;
		flex-direction: column;
		gap: 14px;
		max-height: min(70vh, 640px);
		overflow-y: auto;
		padding: 0 8px 2px;
	}
	.detail__title {
		display: flex;
		align-items: center;
		gap: 12px;
	}
	.detail__lede {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.docs {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		margin-left: 4px;
		border-radius: 4px;
		color: var(--md-sys-color-primary);
		font-weight: 600;
		text-decoration: underline;
		text-underline-offset: 2px;
	}
	.docs:hover {
		text-decoration-thickness: 2px;
	}
	.docs:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.headers {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin: 0;
		padding: 0;
		border: none;
	}
	.headers__legend {
		padding: 0;
		font-weight: 650;
		color: var(--md-sys-color-on-surface);
	}
	.headers__hint {
		margin: 0;
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.header-row {
		display: grid;
		grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) auto;
		align-items: end;
		gap: 8px;
	}
	.error {
		margin: 0;
		color: var(--md-sys-color-error);
	}
	.actions {
		display: flex;
		justify-content: flex-end;
	}
	@media (max-width: 480px) {
		.header-row {
			grid-template-columns: minmax(0, 1fr) auto;
		}
		.header-row :global(.m3-field:nth-child(2)) {
			grid-row: 2;
		}
		.header-row :global(.header-row__remove) {
			grid-row: 1 / span 2;
			align-self: center;
		}
		.actions :global(.m3-button) {
			width: 100%;
			min-height: 48px;
		}
	}
</style>
