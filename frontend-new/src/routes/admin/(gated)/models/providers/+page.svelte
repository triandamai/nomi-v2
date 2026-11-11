<script lang="ts">
	import { PlugZap } from '@lucide/svelte';
	import { enhance } from '$app/forms';
	import { m } from '$lib/paraglide/messages';
	import { can } from '$lib/admin/access';
	import type { Connection } from '$lib/admin/api';
	import PageHeader from '$lib/ds/admin/PageHeader.svelte';
	import FormSection from '$lib/ds/admin/FormSection.svelte';
	import Select from '$lib/ds/admin/Select.svelte';
	import SecretField from '$lib/ds/admin/SecretField.svelte';
	import TextArea from '$lib/ds/admin/TextArea.svelte';
	import ConfirmDialog from '$lib/ds/admin/ConfirmDialog.svelte';
	import Notice from '$lib/ds/admin/Notice.svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import EmptyState from '$lib/ds/lists/EmptyState.svelte';
	import LoadFailed from '$lib/admin/ui/LoadFailed.svelte';
	import Outcome from '$lib/admin/ui/Outcome.svelte';

	// Models → Providers (models: view): each provider connected for everyone, with the models
	// that run on it; its name, key and address changed (write) or the provider disconnected once no
	// model uses it (manage); and connecting another from the catalog (write). Keys are written,
	// never read back.
	let { data, form } = $props();

	const canWrite = $derived(can(data.access, 'models', 'write'));
	const canManage = $derived(can(data.access, 'models', 'manage'));
	const nameOf = (id: string) => data.providers.find((provider) => provider.id === id)?.name ?? id;

	let disconnecting = $state<Connection | null>(null);
	let confirmOpen = $state(false);
	let disconnectForm: HTMLFormElement | undefined = $state();
	const inUse = $derived(form?.action === 'disconnect' && !form.ok && form.problem?.code === 'in_use' && 'models' in form.problem ? (form.problem.models ?? []) : null);
</script>

<PageHeader title={m.admin_page_providers()} intro={m.admin_providers_intro()} />

{#if data.connections.ok}
	{#if data.connections.data.length === 0}
		<EmptyState icon={PlugZap} title={m.admin_providers_empty_title()} body={m.admin_providers_empty_body()} />
	{/if}
	{#each data.connections.data as connection (connection.id)}
		{#snippet actions()}
			{#if canManage}<Button label={m.admin_provider_disconnect()} variant="ghost-danger" onclick={() => ((disconnecting = connection), (confirmOpen = true))} />{/if}
			{#if canWrite}<Button label={m.admin_save()} variant="inverse" type="submit" />{/if}
		{/snippet}
		<form method="POST" action="?/update" use:enhance={() => ({ update }) => update({ reset: false })}>
			<input type="hidden" name="id" value={connection.id} />
			<input type="hidden" name="was_base_url" value={connection.base_url ?? ''} />
			<FormSection title={connection.label} description={nameOf(connection.provider_id)} footer={canWrite || canManage ? actions : undefined}>
				<p class="font-mono text-label text-text-2">
					{connection.models_in_use.length ? m.admin_provider_models_in_use({ models: connection.models_in_use.join(', ') }) : m.admin_provider_unused()}
				</p>
				{#if canWrite}
					<div class="grid gap-4 sm:grid-cols-2">
						<TextField label={m.admin_provider_label()} name="label" value={connection.label} autocomplete="off" />
						<TextField label={m.admin_provider_base_url()} name="base_url" value={connection.base_url ?? ''} mono autocomplete="off" />
						<div class="sm:col-span-2">
							<SecretField
								label={m.admin_provider_key()}
								name="api_key"
								set={Boolean(connection.api_key_masked)}
								status={connection.api_key_masked ? m.admin_model_key_saved({ masked: connection.api_key_masked }) : m.admin_model_key_none()}
							/>
						</div>
					</div>
				{/if}
				{#if form?.action === 'update' && disconnecting === null}<Outcome {form} action="update" done={m.admin_provider_saved()} failed={m.admin_provider_save_failed()} />{/if}
			</FormSection>
		</form>
	{/each}
	{#if inUse}
		<Notice kind="error" text={m.admin_provider_disconnect_failed()} detail={m.admin_provider_in_use({ models: inUse.join(', ') })} />
	{:else}
		<Outcome {form} action="disconnect" done={m.admin_provider_disconnected()} failed={m.admin_provider_disconnect_failed()} />
	{/if}
{:else}
	<LoadFailed title={m.admin_providers_failed()} problem={data.connections.problem} />
{/if}

{#if canWrite && data.providers.length > 0}
	{#snippet connect()}<Button label={m.admin_provider_connect_action()} variant="inverse" type="submit" />{/snippet}
	<form method="POST" action="?/connect" use:enhance={() => ({ update, result }) => update({ reset: result.type === 'success' })}>
		<FormSection title={m.admin_provider_connect()} description={m.admin_provider_connect_desc()} footer={connect}>
			<div class="grid gap-4 sm:grid-cols-2">
				<Select label={m.admin_provider_catalog()} name="provider_id" options={data.providers.map((provider) => ({ value: provider.id, label: provider.name }))} />
				<TextField label={m.admin_provider_label()} name="label" autocomplete="off" />
				<div class="sm:col-span-2"><SecretField label={m.admin_provider_key()} name="api_key" set={false} status={m.admin_model_key_none()} /></div>
				<TextField label={m.admin_provider_base_url()} name="base_url" helper={m.admin_model_base_url_helper()} mono autocomplete="off" />
				<TextArea label={m.admin_provider_headers()} name="headers" helper={m.admin_provider_headers_helper()} />
			</div>
			<Outcome {form} action="connect" done={m.admin_provider_connected()} failed={m.admin_provider_connect_failed()} />
		</FormSection>
	</form>
{/if}

<form bind:this={disconnectForm} method="POST" action="?/disconnect" use:enhance={() => async ({ update }) => {
		await update();
		disconnecting = null;
	}} hidden>
	<input type="hidden" name="id" value={disconnecting?.id ?? ''} />
</form>
<ConfirmDialog
	bind:open={confirmOpen}
	title={m.admin_provider_disconnect_title({ provider: disconnecting?.label ?? '' })}
	body={m.admin_provider_disconnect_body()}
	confirmLabel={m.admin_provider_disconnect()}
	cancelLabel={m.admin_cancel()}
	onconfirm={() => {
		confirmOpen = false;
		disconnectForm?.requestSubmit();
	}}
/>
