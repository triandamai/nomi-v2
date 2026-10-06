<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { deserialize } from '$app/forms';
	import { enhance } from '$app/forms';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import IconChip from '$lib/components/icons/IconChip.svelte';
	import IconPlus from '$lib/components/icons/IconPlus.svelte';
	import Select from '$lib/components/m3/Select.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import { formatUsd } from '$lib/usage';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	const PROVIDER_OPTIONS = [
		{ value: 'anthropic', label: 'Anthropic' },
		{ value: 'openai', label: 'OpenAI' },
		{ value: 'openrouter', label: 'OpenRouter' },
		{ value: 'gemini', label: 'Gemini' },
		{ value: 'deepseek', label: 'DeepSeek' },
		{ value: 'fake', label: m.key_fake() },
	];

	const providerName = (value: string) => PROVIDER_OPTIONS.find((o) => o.value === value)?.label ?? value;

	let sheetOpen = $state(false);
	let editingId = $state<string | null>(null);

	let label = $state('');
	let provider = $state('anthropic');
	let apiKey = $state('');
	let baseUrl = $state('');
	let modelId = $state('');
	let inputPrice = $state('');
	let outputPrice = $state('');

	let fetching = $state(false);
	let fetchedModels = $state<{ id: string; label: string | null }[]>([]);
	let modelEntryMode = $state<'select' | 'manual'>('manual');
	let fetchError = $state<string | null>(null);

	function resetSheetState() {
		fetchedModels = [];
		modelEntryMode = 'manual';
		fetchError = null;
		fetching = false;
	}

	function openCreate() {
		editingId = null;
		label = '';
		provider = 'anthropic';
		apiKey = '';
		baseUrl = '';
		modelId = '';
		inputPrice = '';
		outputPrice = '';
		resetSheetState();
		sheetOpen = true;
	}

	function openEdit(model: PageData['models'][number]) {
		editingId = model.id;
		label = model.label;
		provider = model.provider;
		apiKey = '';
		baseUrl = model.base_url ?? '';
		modelId = model.model_id;
		inputPrice = model.input_usd_per_mtok?.toString() ?? '';
		outputPrice = model.output_usd_per_mtok?.toString() ?? '';
		resetSheetState();
		sheetOpen = true;
	}

	async function fetchModels() {
		fetching = true;
		fetchError = null;

		const body = new FormData();
		body.set('provider', provider);
		body.set('api_key', apiKey);
		body.set('base_url', baseUrl);
		body.set('existing_model_id', editingId ?? '');

		const response = await fetch('?/fetchModels', { method: 'POST', body });
		const result = deserialize(await response.text());
		fetching = false;

		if (result.type === 'success' && result.data?.models) {
			fetchedModels = result.data.models as { id: string; label: string | null }[];
			modelEntryMode = 'select';
			if (fetchedModels.length > 0 && !fetchedModels.some((m) => m.id === modelId)) {
				modelId = fetchedModels[0].id;
			}
			if (fetchedModels.length === 0) {
				fetchError = m.key_no_models();
				modelEntryMode = 'manual';
			}
		} else {
			fetchedModels = [];
			modelEntryMode = 'manual';
			fetchError =
				(result.type === 'failure' && (result.data?.error as string)) ||
				m.err_fetch_models();
		}
	}
</script>

<PageHeader title={m.admin_models()} lede={m.llm_lede()} agent="coding">
	{#snippet actions()}
		<Button type="button" variant="filled" onclick={openCreate}><IconPlus size={18} /> {m.llm_add()}</Button>
	{/snippet}
</PageHeader>

{#if data.models.length === 0}
	<div class="empty">
		<span class="model__icon"><IconChip size={26} /></span>
		<p class="empty__text"><strong>{m.llm_empty_title()}</strong> {m.llm_empty()}</p>
	</div>
{:else}
	<ul class="models">
		{#each data.models as model (model.id)}
			<li class="model" class:model--default={model.is_default}>
				<div class="model__top">
					<span class="model__icon"><IconChip size={24} /></span>
					<div class="model__text">
						<h3 class="model__name">{model.label}</h3>
						<p class="model__provider">{providerName(model.provider)}</p>
					</div>
					{#if model.is_default}<span class="chip">{m.llm_default()}</span>{/if}
				</div>
				<dl class="model__facts">
					<div><dt>{m.key_model()}</dt><dd>{model.model_id}</dd></div>
					<div><dt>{m.llm_price()}</dt><dd>{model.input_usd_per_mtok != null || model.output_usd_per_mtok != null ? m.llm_price_value({ input: formatUsd(model.input_usd_per_mtok ?? 0), output: formatUsd(model.output_usd_per_mtok ?? 0) }) : m.llm_price_unset()}</dd></div>
					<div><dt>{m.llm_key()}</dt><dd>{model.api_key_masked || m.llm_none()}</dd></div>
				</dl>
				<div class="model__actions">
					<Button type="button" variant="tonal" size="xs" onclick={() => openEdit(model)}>{m.dyn_edit()}</Button>
					{#if !model.is_default}
						<form method="POST" action="?/setDefault" use:enhance>
							<input type="hidden" name="id" value={model.id} />
							<Button type="submit" variant="text" size="xs">{m.llm_make_default()}</Button>
						</form>
						<form method="POST" action="?/delete" use:enhance>
							<input type="hidden" name="id" value={model.id} />
							<Button type="submit" variant="text" size="xs" class="danger">{m.common_delete()}</Button>
						</form>
					{/if}
				</div>
			</li>
		{/each}
	</ul>
{/if}

<BottomSheet bind:open={sheetOpen}>
	<h2 class="sheet-title">{editingId ? m.llm_edit_named({ label: label || m.llm_model() }) : m.llm_add()}</h2>

	{#if form?.error}
		<p class="md-body-medium mt-2" style="color: var(--md-sys-color-error)">{form.error}</p>
	{/if}

	<form
		method="POST"
		action={editingId ? '?/update' : '?/create'}
		use:enhance={() => {
			return async ({ result, update }) => {
				await update({ reset: false });
				if (result.type === 'success') {
					sheetOpen = false;
				}
			};
		}}
		class="mt-4 flex flex-col gap-3"
	>
		{#if editingId}
			<input type="hidden" name="id" value={editingId} />
		{/if}
		<TextField id="label" name="label" label={m.key_label()} bind:value={label} required />
		<Select label={m.key_provider()} name="provider" bind:value={provider} options={PROVIDER_OPTIONS} />
		<TextField
			id="api_key"
			name="api_key"
			type="password"
			label={m.key_api_key()}
			bind:value={apiKey}
			placeholder={editingId ? m.llm_keep_key() : undefined}
		/>
		<TextField id="base_url" name="base_url" label={m.key_base_url()} bind:value={baseUrl} />

		<Button
			type="button"
			variant="outlined"
			class="w-fit"
			disabled={fetching || (provider !== 'fake' && !apiKey && !editingId)}
			onclick={fetchModels}
		>
			{fetching ? m.key_fetching() : m.key_fetch()}
		</Button>
		{#if fetchError}
			<p class="md-body-small" style="color: var(--md-sys-color-on-surface-variant)">{fetchError}</p>
		{/if}

		{#if modelEntryMode === 'select' && fetchedModels.length > 0}
			<Select
				label={m.key_model()}
				name="model_id"
				bind:value={modelId}
				options={fetchedModels.map((m) => ({ value: m.id, label: m.label ? `${m.id} (${m.label})` : m.id }))}
			/>
			<button
				type="button"
				class="md-label-large self-start"
				style="color: var(--md-sys-color-primary); background: none; border: none; cursor: pointer; padding: 0"
				onclick={() => (modelEntryMode = 'manual')}
			>
				{m.key_manual()}
			</button>
		{:else}
			<TextField id="model_id" name="model_id" label={m.key_model_id()} bind:value={modelId} required />
			{#if fetchedModels.length > 0}
				<button
					type="button"
					class="md-label-large self-start"
					style="color: var(--md-sys-color-primary); background: none; border: none; cursor: pointer; padding: 0"
					onclick={() => (modelEntryMode = 'select')}
				>
					{m.key_choose_list()}
				</button>
			{/if}
		{/if}

		<fieldset class="prices">
			<legend class="prices__legend">{m.llm_prices()}</legend>
			<p class="prices__hint">{m.llm_prices_hint()}</p>
			<div class="prices__row">
				<TextField id="input_usd_per_mtok" name="input_usd_per_mtok" label={m.llm_price_in()} type="number" inputmode="decimal" min="0" step="0.0001" bind:value={inputPrice} />
				<TextField id="output_usd_per_mtok" name="output_usd_per_mtok" label={m.llm_price_out()} type="number" inputmode="decimal" min="0" step="0.0001" bind:value={outputPrice} />
			</div>
		</fieldset>
		<div class="flex gap-2 pt-2">
			<Button type="submit" variant="filled">{editingId ? m.common_save() : m.llm_add()}</Button>
			<Button type="button" variant="outlined" onclick={() => (sheetOpen = false)}>{m.common_cancel()}</Button>
		</div>
	</form>
</BottomSheet>

<style>
	.prices {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin: 4px 0 0;
		padding: 0;
		border: none;
	}
	.prices__legend {
		padding: 0;
		font-weight: 700;
		color: var(--md-sys-color-on-surface);
	}
	.prices__hint {
		margin: 0;
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.prices__row {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
		gap: 12px;
	}
	.empty {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 20px 24px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
	}
	.empty__text {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.empty__text strong {
		color: var(--md-sys-color-on-surface);
	}
	.models {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(min(100%, 340px), 1fr));
		gap: 12px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.model {
		display: flex;
		flex-direction: column;
		gap: 14px;
		padding: 20px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
	}
	.model--default {
		box-shadow: inset 0 0 0 2px var(--md-sys-color-primary);
	}
	.model__top {
		display: flex;
		align-items: center;
		gap: 14px;
	}
	.model__icon {
		display: grid;
		flex: none;
		place-items: center;
		width: 48px;
		height: 48px;
		border-radius: 16px;
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.model__text {
		flex: 1;
		min-width: 0;
	}
	.model__name {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.25rem;
		font-weight: 700;
		overflow-wrap: anywhere;
	}
	.model__provider {
		margin: 2px 0 0;
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.chip {
		flex: none;
		padding: 3px 12px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
		font-size: 0.8125rem;
		font-weight: 650;
	}
	.model__facts {
		display: flex;
		flex-direction: column;
		gap: 6px;
		margin: 0;
		padding: 12px 14px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container);
	}
	.model__facts div {
		display: flex;
		justify-content: space-between;
		gap: 12px;
	}
	.model__facts dt {
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.6875rem;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--md-sys-color-on-surface-variant);
	}
	.model__facts dd {
		margin: 0;
		min-width: 0;
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.8125rem;
		text-align: right;
		overflow-wrap: anywhere;
	}
	.model__actions {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
	}
	.model__actions :global(.danger) {
		color: var(--md-sys-color-error);
	}
	.sheet-title {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.5rem;
		font-weight: 700;
		color: var(--md-sys-color-on-surface);
	}
</style>
