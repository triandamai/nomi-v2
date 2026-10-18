<script lang="ts">
	// Picking a model from one of the person's own provider connections (Model page and chat
	// page). Without a connection it connects one in place first, through the page's `connect`
	// action, then carries on to the model. The page needs `connect`, `connectionModels` and
	// `selectConnectionModel` actions (see $lib/server/providers).
	import { untrack } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import Select from '$lib/components/m3/Select.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import ConnectProviderDialog from '$lib/components/providers/ConnectProviderDialog.svelte';
	import { postAction, reloadData } from '$lib/providerActions';
	import {
		connectInputToForm,
		connectionErrorMessage,
		connectionName,
		type ConnectInput,
		type ConnectionModel,
		type ProviderConnection,
		type ProviderDefinition,
	} from '$lib/providers';
	import type { LlmUserSelection } from '$lib/types';

	let {
		open = $bindable(false),
		connections,
		providers = [],
		selection = null,
		onSaved,
	}: {
		open?: boolean;
		connections: ProviderConnection[];
		/** The catalog, when the page has it; otherwise it's fetched on first open. */
		providers?: ProviderDefinition[];
		selection?: LlmUserSelection | null;
		onSaved?: () => void;
	} = $props();

	let fetchedCatalog = $state<ProviderDefinition[]>([]);
	const catalog = $derived(providers.length > 0 ? providers : fetchedCatalog);
	let catalogRequested = false;
	async function ensureCatalog() {
		if (providers.length > 0 || catalogRequested) return;
		catalogRequested = true;
		try {
			const response = await fetch('/providers/catalog');
			if (response.ok) fetchedCatalog = ((await response.json()) as { providers: ProviderDefinition[] }).providers;
			else catalogRequested = false;
		} catch {
			catalogRequested = false;
		}
	}

	const connectionOptions = $derived(connections.map((c) => ({ value: c.id, label: connectionName(c, catalog) })));

	let connectionId = $state('');
	let modelId = $state('');
	let label = $state('');
	let models = $state<ConnectionModel[]>([]);
	let entry = $state<'select' | 'manual'>('manual');
	let loadingModels = $state(false);
	let modelsError = $state<string | null>(null);
	let saving = $state(false);
	let saveError = $state<string | null>(null);

	// Each opening starts on the connection in use (or the first one), nothing typed yet.
	let wasOpen = false;
	$effect(() => {
		const opening = open && !wasOpen;
		wasOpen = open;
		if (!opening) return;
		untrack(() => {
			const current = selection?.kind === 'connection' && connections.some((c) => c.id === selection.connection_id) ? selection : null;
			connectionId = current?.connection_id ?? connections[0]?.id ?? '';
			modelId = current?.model_id ?? '';
			label = '';
			saveError = null;
			void ensureCatalog();
		});
	});

	// The chosen connection's models, asked of the provider whenever it changes.
	let request = 0;
	$effect(() => {
		if (!open || !connectionId) return;
		const id = connectionId;
		untrack(() => void loadModels(id));
	});

	async function loadModels(id: string) {
		const mine = ++request;
		loadingModels = true;
		modelsError = null;
		const body = new FormData();
		body.set('connection_id', id);
		const outcome = await postAction<{ connectionModels: ConnectionModel[] }>('connectionModels', body);
		if (mine !== request) return;
		loadingModels = false;
		models = outcome.ok ? outcome.data.connectionModels : [];
		if (!outcome.ok) modelsError = connectionErrorMessage(outcome.error);
		else if (models.length === 0) modelsError = m.key_no_models();
		entry = models.length > 0 ? 'select' : 'manual';
		if (models.length > 0 && !models.some((model) => model.id === modelId)) modelId = models[0].id;
	}

	async function save(event: SubmitEvent) {
		event.preventDefault();
		if (!connectionId || !modelId.trim() || saving) return;
		saving = true;
		saveError = null;
		const body = new FormData();
		body.set('connection_id', connectionId);
		body.set('model_id', modelId.trim());
		const name = label.trim() || models.find((model) => model.id === modelId.trim())?.label || '';
		if (name) body.set('label', name);
		const outcome = await postAction('selectConnectionModel', body);
		saving = false;
		if (outcome.ok) {
			await reloadData();
			open = false;
			onSaved?.();
		} else {
			saveError = connectionErrorMessage(outcome.error);
		}
	}

	// Connecting in place.
	let connectOpen = $state(false);
	let connecting = $state(false);
	let connectError = $state<string | null>(null);

	function openConnect() {
		connectError = null;
		void ensureCatalog();
		connectOpen = true;
	}

	async function connect(input: ConnectInput) {
		connecting = true;
		connectError = null;
		const outcome = await postAction<{ connection: ProviderConnection }>('connect', connectInputToForm(input));
		if (outcome.ok) {
			await reloadData();
			// Straight on to its models.
			connectionId = outcome.data.connection.id;
			modelId = '';
			connectOpen = false;
		} else {
			connectError = connectionErrorMessage(outcome.error, outcome.models);
		}
		connecting = false;
	}
</script>

<BottomSheet bind:open>
	<h2 class="md-headline-small-emphasized title">{m.prov_sheet_title()}</h2>

	{#if connections.length === 0}
		<p class="hint">{m.prov_sheet_none()}</p>
		<div class="actions">
			<Button type="button" variant="filled" onclick={openConnect}>{m.prov_connect_cta()}</Button>
			<Button type="button" variant="outlined" onclick={() => (open = false)}>{m.common_cancel()}</Button>
		</div>
	{:else}
		<form class="form" onsubmit={save}>
			<Select label={m.key_provider()} name="connection_id" bind:value={connectionId} options={connectionOptions} />

			{#if loadingModels}
				<p class="hint" role="status">{m.prov_models_loading()}</p>
			{:else if modelsError}
				<p class="hint">{modelsError}</p>
			{/if}

			{#if entry === 'select' && models.length > 0}
				<Select
					label={m.key_model()}
					name="model_id"
					bind:value={modelId}
					options={models.map((model) => ({ value: model.id, label: model.label ? `${model.label} (${model.id})` : model.id }))}
				/>
				<button type="button" class="link" onclick={() => (entry = 'manual')}>{m.key_manual()}</button>
			{:else}
				<TextField name="model_id" label={m.key_model_id()} bind:value={modelId} type="text" autocomplete="off" spellcheck="false" required />
				{#if models.length > 0}
					<button type="button" class="link" onclick={() => (entry = 'select')}>{m.key_choose_list()}</button>
				{/if}
			{/if}

			<TextField label={m.prov_label_optional()} bind:value={label} autocomplete="off" />

			{#if saveError}
				<p class="error" role="alert">{saveError}</p>
			{/if}

			<div class="actions">
				<Button type="submit" variant="filled" disabled={!modelId.trim() || saving}>{saving ? m.prov_saving() : m.prov_use_model()}</Button>
				<Button type="button" variant="outlined" onclick={() => (open = false)}>{m.common_cancel()}</Button>
			</div>
			<button type="button" class="link" onclick={openConnect}>{m.prov_connect_another()}</button>
		</form>
	{/if}
</BottomSheet>

<ConnectProviderDialog
	bind:open={connectOpen}
	providers={catalog}
	connectedIds={connections.map((c) => c.provider_id)}
	error={connectError}
	busy={connecting}
	onsubmit={connect}
/>

<style>
	.title {
		margin: 0;
		color: var(--md-sys-color-on-surface);
	}
	.hint {
		margin: 8px 0 0;
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.form {
		display: flex;
		flex-direction: column;
		gap: 12px;
		margin-top: 16px;
	}
	.form .hint {
		margin: 0;
	}
	.error {
		margin: 0;
		color: var(--md-sys-color-error);
	}
	.actions {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		margin-top: 16px;
	}
	.form .actions {
		margin-top: 4px;
	}
	.link {
		align-self: flex-start;
		min-height: 32px;
		padding: 0 4px;
		border: none;
		border-radius: 6px;
		background: none;
		color: var(--md-sys-color-primary);
		font: inherit;
		font-weight: 600;
		cursor: pointer;
	}
	.link:hover {
		text-decoration: underline;
	}
	.link:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	@media (max-width: 640px) {
		.link {
			min-height: 44px;
		}
	}
</style>
