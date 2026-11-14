<script lang="ts">
	import { Search } from '@lucide/svelte';
	import { enhance } from '$app/forms';
	import { m } from '$lib/paraglide/messages';
	import { can } from '$lib/admin/access';
	import type { FetchedModel } from '$lib/admin/api';
	import { EMBEDDING_PROVIDERS } from '$lib/admin/models';
	import PageHeader from '$lib/ds/admin/PageHeader.svelte';
	import FormSection from '$lib/ds/admin/FormSection.svelte';
	import Select from '$lib/ds/admin/Select.svelte';
	import SecretField from '$lib/ds/admin/SecretField.svelte';
	import Notice from '$lib/ds/admin/Notice.svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import LoadFailed from '$lib/admin/ui/LoadFailed.svelte';
	import Outcome from '$lib/admin/ui/Outcome.svelte';

	// Models → Embedding (models: view; write changes it): the model memory and search use, its
	// provider, key (written, never read back) and address, and finding what the provider offers.
	let { data, form } = $props();

	const canWrite = $derived(can(data.access, 'models', 'write'));
	const initial = () => (data.embedding.ok ? (data.embedding.data?.model_id ?? '') : '');
	let modelId = $state(initial());
	let mainForm: HTMLFormElement | undefined = $state();
	let findButton: HTMLButtonElement | undefined = $state();
	const found = $derived(form?.action === 'findModels' && form.ok ? (form.data as { models: FetchedModel[] }).models : null);
</script>

{#snippet save()}<Button label={m.admin_save()} variant="inverse" type="submit" />{/snippet}

<PageHeader title={m.admin_page_embedding()} intro={m.admin_embedding_intro()} />

{#if data.embedding.ok}
	{@const current = data.embedding.data ?? { provider: 'openai', model_id: '', base_url: null, api_key_masked: '' }}
	{#if !data.embedding.data}<Notice kind="info" text={m.admin_embedding_unsaved()} />{/if}
	<form bind:this={mainForm} method="POST" action="?/save" use:enhance={() => ({ update }) => update({ reset: false })}>
		<button bind:this={findButton} type="submit" formaction="?/findModels" hidden tabindex="-1" aria-hidden="true"></button>
		<FormSection title={m.admin_embedding_section()} footer={canWrite ? save : undefined}>
			<div class="grid gap-4 sm:grid-cols-2">
				<Select label={m.admin_embedding_provider()} name="provider" value={current.provider} options={EMBEDDING_PROVIDERS.map((provider) => ({ value: provider, label: provider }))} mono disabled={!canWrite} />
				<TextField label={m.admin_model_base_url()} name="base_url" value={current.base_url ?? ''} helper={m.admin_model_base_url_helper()} mono disabled={!canWrite} autocomplete="off" />
				<div class="sm:col-span-2">
					<SecretField
						label={m.admin_model_key()}
						name="api_key"
						set={Boolean(current.api_key_masked)}
						status={current.api_key_masked ? m.admin_model_key_saved({ masked: current.api_key_masked }) : m.admin_model_key_none()}
						disabled={!canWrite}
					/>
				</div>
				<div class="flex flex-wrap items-end gap-3 sm:col-span-2">
					<div class="min-w-60 flex-1"><TextField label={m.admin_model_id()} name="model_id" bind:value={modelId} required mono disabled={!canWrite} autocomplete="off" /></div>
					{#if canWrite}<Button label={m.admin_model_find()} variant="secondary" iconStart={Search} onclick={() => mainForm?.requestSubmit(findButton)} />{/if}
				</div>
				{#if found}
					<div class="sm:col-span-2">
						{#if found.length === 0}
							<Notice kind="info" text={m.admin_model_found_none()} />
						{:else}
							<Select
								label={m.admin_model_found()}
								helper={m.admin_model_found_pick()}
								options={[{ value: '', label: '—' }, ...found.map((one) => ({ value: one.id, label: one.id }))]}
								value=""
								mono
								onchange={(value) => {
									if (value) modelId = value;
								}}
							/>
						{/if}
					</div>
				{/if}
			</div>
			<Outcome {form} action="findModels" failed={m.admin_model_find_failed()} />
			<Outcome {form} action="save" done={m.admin_embedding_saved()} failed={m.admin_embedding_save_failed()} />
		</FormSection>
	</form>
{:else}
	<LoadFailed title={m.admin_embedding_failed()} problem={data.embedding.problem} />
{/if}
