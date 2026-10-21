<script lang="ts">
	// Manage models on a chat page, wired to the page's form actions: `setModelVisibility` saves
	// the switches, and `connect` adds a provider in place (the catalog is fetched the first time),
	// after which the page's data, and so the models, reload.
	import ConnectProviderDialog from '$lib/components/providers/ConnectProviderDialog.svelte';
	import ManageModelsDialog from './ManageModelsDialog.svelte';
	import { postAction, reloadData } from '$lib/providerActions';
	import { connectInputToForm, connectionErrorMessage, type ConnectInput, type ProviderDefinition } from '$lib/providers';
	import type { ModelChange, ModelGroup } from '$lib/modelPicker';

	let {
		open = $bindable(false),
		groups,
		onsaved,
	}: {
		open?: boolean;
		groups: ModelGroup[];
		/** The switches saved: the page updates its picker. */
		onsaved?: (changes: ModelChange[]) => void;
	} = $props();

	async function save(changes: ModelChange[]): Promise<boolean> {
		const body = new FormData();
		body.set('changes', JSON.stringify(changes));
		const outcome = await postAction('setModelVisibility', body);
		if (outcome.ok) onsaved?.(changes);
		return outcome.ok;
	}

	let catalog = $state<ProviderDefinition[]>([]);
	let catalogRequested = false;
	async function ensureCatalog() {
		if (catalogRequested) return;
		catalogRequested = true;
		try {
			const response = await fetch('/providers/catalog');
			if (response.ok) catalog = ((await response.json()) as { providers: ProviderDefinition[] }).providers;
			else catalogRequested = false;
		} catch {
			catalogRequested = false;
		}
	}

	let connectOpen = $state(false);
	let connecting = $state(false);
	let connectError = $state<string | null>(null);
	const connectedIds = $derived(groups.filter((g) => g.source === 'connection').map((g) => g.provider_id));

	function openConnect() {
		connectError = null;
		void ensureCatalog();
		connectOpen = true;
	}

	async function connect(input: ConnectInput) {
		connecting = true;
		connectError = null;
		const outcome = await postAction('connect', connectInputToForm(input));
		if (outcome.ok) {
			await reloadData();
			connectOpen = false;
		} else {
			connectError = connectionErrorMessage(outcome.error, outcome.models);
		}
		connecting = false;
	}
</script>

<ManageModelsDialog bind:open {groups} onchange={save} onconnect={openConnect} />
<ConnectProviderDialog bind:open={connectOpen} providers={catalog} {connectedIds} error={connectError} busy={connecting} onsubmit={connect} />
