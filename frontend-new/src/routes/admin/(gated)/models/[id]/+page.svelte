<script lang="ts">
	import { Search } from '@lucide/svelte';
	import { enhance } from '$app/forms';
	import { m } from '$lib/paraglide/messages';
	import { can } from '$lib/admin/access';
	import type { FetchedModel } from '$lib/admin/api';
	import { MEDIA_KINDS, OWN_KEY_PROVIDERS } from '$lib/admin/models';
	import PageHeader from '$lib/ds/admin/PageHeader.svelte';
	import FormSection from '$lib/ds/admin/FormSection.svelte';
	import Select from '$lib/ds/admin/Select.svelte';
	import SecretField from '$lib/ds/admin/SecretField.svelte';
	import ConfirmDialog from '$lib/ds/admin/ConfirmDialog.svelte';
	import Notice from '$lib/ds/admin/Notice.svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';
	import Segmented from '$lib/ds/inputs/Segmented.svelte';
	import Checkbox from '$lib/ds/inputs/Checkbox.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import LoadFailed from '$lib/admin/ui/LoadFailed.svelte';
	import Outcome from '$lib/admin/ui/Outcome.svelte';

	// Models → a model (models: write; view shows it): the name people see, where it runs (a
	// provider connected for everyone, or its own provider and key), its id, found from what the
	// provider serves, its prices per million tokens, and what it opens. Delete needs manage.
	let { data, form } = $props();

	const canWrite = $derived(can(data.access, 'models', 'write'));
	const canManage = $derived(can(data.access, 'models', 'manage'));
	const existing = $derived(data.model?.ok ? data.model.data : null);

	const initial = () => (data.model?.ok ? data.model.data : null);
	// An existing model stays where it runs; a new one starts on a connected provider when there is one.
	const firstSource = () => (initial() ? (initial()!.connection_id ? 'connection' : 'own') : data.connections.length > 0 ? 'connection' : 'own');
	let source = $state(firstSource());
	let modelId = $state(initial()?.model_id ?? '');
	let mediaMode = $state(initial()?.media_inputs ? 'listed' : 'auto');
	let deleting = $state(false);
	let deleteForm: HTMLFormElement | undefined = $state();
	let mainForm: HTMLFormElement | undefined = $state();
	let findButton: HTMLButtonElement | undefined = $state();

	const sources = [
		{ value: 'connection', label: m.admin_model_source_connection() },
		{ value: 'own', label: m.admin_model_source_own() },
	];
	const mediaModes = [
		{ value: 'auto', label: m.admin_model_media_auto() },
		{ value: 'listed', label: m.admin_model_media_listed() },
	];
	const MEDIA_LABELS = { image: m.admin_model_media_image, pdf: m.admin_model_media_pdf, audio: m.admin_model_media_audio, video: m.admin_model_media_video };
	const found = $derived(form?.action === 'findModels' && form.ok ? (form.data as { models: FetchedModel[] }).models : null);
</script>

{#snippet save()}<Button label={existing ? m.admin_save() : m.admin_model_create()} variant="inverse" type="submit" />{/snippet}

{#if data.model && !data.model.ok}
	<PageHeader title={m.admin_page_models()} back={{ href: '/admin/models', label: m.admin_model_back() }} />
	<LoadFailed title={m.admin_model_failed()} problem={data.model.problem} />
{:else}
	<PageHeader title={existing?.label ?? m.admin_model_new()} meta={existing?.model_id} back={{ href: '/admin/models', label: m.admin_model_back() }} />

	<form bind:this={mainForm} method="POST" action="?/save" use:enhance={() => ({ update }) => update({ reset: false })} class="flex flex-col gap-6">
		<!-- "Find models" sends this same form to its own action. -->
		<button bind:this={findButton} type="submit" formaction="?/findModels" hidden tabindex="-1" aria-hidden="true"></button>

		<FormSection title={m.admin_model_section()} description={m.admin_model_section_desc()}>
			<div class="flex flex-col gap-4">
				<TextField label={m.admin_model_label()} name="label" value={existing?.label ?? ''} required disabled={!canWrite} autocomplete="off" />
				<Segmented label={m.admin_model_source()} options={sources} bind:value={source} />
				<input type="hidden" name="source" value={source} />
				{#if source === 'connection'}
					{#if data.connections.length === 0}
						<Notice kind="info" text={m.admin_model_no_connections()} />
					{:else}
						<Select
							label={m.admin_model_connection()}
							name="connection_id"
							value={existing?.connection_id ?? data.connections[0].id}
							options={data.connections.map((connection) => ({ value: connection.id, label: `${connection.label} · ${connection.provider_id}` }))}
							disabled={!canWrite}
						/>
					{/if}
				{:else}
					<div class="grid gap-4 sm:grid-cols-2">
						<Select
							label={m.admin_model_provider()}
							name="provider"
							value={existing?.provider ?? OWN_KEY_PROVIDERS[0]}
							options={OWN_KEY_PROVIDERS.map((provider) => ({ value: provider, label: provider }))}
							mono
							disabled={!canWrite}
						/>
						<TextField label={m.admin_model_base_url()} name="base_url" value={existing?.base_url ?? ''} helper={m.admin_model_base_url_helper()} mono disabled={!canWrite} autocomplete="off" />
					</div>
					<SecretField
						label={m.admin_model_key()}
						name="api_key"
						set={Boolean(existing?.api_key_masked)}
						status={existing?.api_key_masked ? m.admin_model_key_saved({ masked: existing.api_key_masked }) : m.admin_model_key_none()}
						disabled={!canWrite}
					/>
				{/if}
				<div class="flex flex-wrap items-end gap-3">
					<div class="min-w-60 flex-1"><TextField label={m.admin_model_id()} name="model_id" bind:value={modelId} required mono disabled={!canWrite} autocomplete="off" /></div>
					{#if canWrite}<Button label={m.admin_model_find()} variant="secondary" iconStart={Search} onclick={() => mainForm?.requestSubmit(findButton)} />{/if}
				</div>
				{#if found}
					{#if found.length === 0}
						<Notice kind="info" text={m.admin_model_found_none()} />
					{:else}
						<Select
							label={m.admin_model_found()}
							helper={m.admin_model_found_pick()}
							options={[{ value: '', label: '—' }, ...found.map((one) => ({ value: one.id, label: one.label ? `${one.label} · ${one.id}` : one.id }))]}
							value=""
							mono
							onchange={(value) => {
								if (value) modelId = value;
							}}
						/>
					{/if}
				{/if}
				<Outcome {form} action="findModels" failed={m.admin_model_find_failed()} />
			</div>
		</FormSection>

		<FormSection title={m.admin_model_prices()} description={m.admin_model_prices_desc()}>
			<div class="grid gap-4 sm:grid-cols-3">
				<TextField label={m.admin_model_price_in()} name="input_usd_per_mtok" value={existing?.input_usd_per_mtok?.toString() ?? ''} placeholder="0.27" mono disabled={!canWrite} autocomplete="off" />
				<TextField label={m.admin_model_price_out()} name="output_usd_per_mtok" value={existing?.output_usd_per_mtok?.toString() ?? ''} placeholder="1.10" mono disabled={!canWrite} autocomplete="off" />
				<TextField label={m.admin_model_price_cached()} name="cached_input_usd_per_mtok" value={existing?.cached_input_usd_per_mtok?.toString() ?? ''} mono disabled={!canWrite} autocomplete="off" />
			</div>
		</FormSection>

		<FormSection title={m.admin_model_media()} footer={canWrite ? save : undefined}>
			<div class="flex flex-col gap-3">
				<Segmented label={m.admin_model_media()} options={mediaModes} bind:value={mediaMode} />
				<input type="hidden" name="media_mode" value={mediaMode} />
				{#if mediaMode === 'listed'}
					<div class="flex flex-wrap gap-4">
						{#each MEDIA_KINDS as kind (kind)}
							<Checkbox label={MEDIA_LABELS[kind]()} name="media_{kind}" checked={existing?.media_inputs?.includes(kind) ?? false} disabled={!canWrite} />
						{/each}
					</div>
				{/if}
			</div>
			<Outcome {form} action="save" done={m.admin_model_saved()} failed={m.admin_model_save_failed()} />
		</FormSection>
	</form>

	{#if existing && canManage}
		{#snippet remove()}<Button label={m.admin_model_delete()} variant="danger" onclick={() => (deleting = true)} />{/snippet}
		<FormSection title={m.admin_model_delete_section()} description={m.admin_model_delete_desc()} footer={remove} danger>
			<Outcome {form} action="delete" failed={m.admin_model_delete_failed()} />
		</FormSection>
		<form bind:this={deleteForm} method="POST" action="?/delete" use:enhance hidden></form>
		<ConfirmDialog
			bind:open={deleting}
			title={m.admin_model_delete_title({ model: existing.label })}
			body={m.admin_model_delete_body()}
			confirmLabel={m.admin_model_delete()}
			cancelLabel={m.admin_cancel()}
			onconfirm={() => {
				deleting = false;
				deleteForm?.requestSubmit();
			}}
		/>
	{/if}
{/if}
