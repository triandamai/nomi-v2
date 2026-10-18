<script lang="ts">
	// The body of both Providers pages (a person's own, and the admin's shared ones): what's
	// connected, what's popular, and the connect dialog. The page's `connect` and `disconnect`
	// actions do the work; the list reloads after each.
	import { m } from '$lib/paraglide/messages';
	import Snackbar from '$lib/components/m3/Snackbar.svelte';
	import ConnectProviderDialog from './ConnectProviderDialog.svelte';
	import ProvidersPanel from './ProvidersPanel.svelte';
	import { postAction, reloadData } from '$lib/providerActions';
	import {
		connectInputToForm,
		connectionErrorMessage,
		connectionName,
		type ConnectInput,
		type ProviderConnection,
		type ProviderDefinition,
	} from '$lib/providers';

	let {
		providers,
		connections,
		loadFailed = false,
	}: { providers: ProviderDefinition[]; connections: ProviderConnection[]; loadFailed?: boolean } = $props();

	let dialogOpen = $state(false);
	let initialProviderId = $state<string | null>(null);
	let busy = $state(false);
	let error = $state<string | null>(null);

	let snackbarOpen = $state(false);
	let snackbarMessage = $state('');
	function toast(message: string) {
		snackbarMessage = message;
		snackbarOpen = true;
	}

	function openDialog(provider: ProviderDefinition | null) {
		initialProviderId = provider?.id ?? null;
		error = null;
		dialogOpen = true;
	}

	async function connect(input: ConnectInput) {
		busy = true;
		error = null;
		const outcome = await postAction<{ connection: ProviderConnection }>('connect', connectInputToForm(input));
		if (outcome.ok) {
			await reloadData();
			dialogOpen = false;
			toast(m.prov_connected_toast({ name: connectionName(outcome.data.connection, providers) }));
		} else {
			error = connectionErrorMessage(outcome.error, outcome.models);
		}
		busy = false;
	}

	async function disconnect(connection: ProviderConnection) {
		const body = new FormData();
		body.set('id', connection.id);
		const outcome = await postAction('disconnect', body);
		if (outcome.ok) {
			await reloadData();
			toast(m.prov_disconnected_toast({ name: connectionName(connection, providers) }));
		} else {
			toast(connectionErrorMessage(outcome.error, outcome.models));
		}
	}
</script>

{#if loadFailed}
	<p class="notice" role="alert">{m.prov_load_failed()}</p>
{/if}

<ProvidersPanel {connections} {providers} onconnect={openDialog} ondisconnect={disconnect} onbrowse={() => openDialog(null)} />

<ConnectProviderDialog
	bind:open={dialogOpen}
	{providers}
	connectedIds={connections.map((c) => c.provider_id)}
	{initialProviderId}
	{error}
	{busy}
	onsubmit={connect}
/>

<Snackbar bind:open={snackbarOpen} message={snackbarMessage} />

<style>
	.notice {
		margin: 0 0 16px;
		color: var(--md-sys-color-error);
	}
</style>
