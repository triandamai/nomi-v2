<script lang="ts">
	import { untrack } from 'svelte';
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
	import Money from '$lib/components/Money.svelte';
	import { formatUsd } from '$lib/usage';
	import { postAction } from '$lib/providerActions';
	import { connectionErrorMessage, connectionName, prefillFromCatalog, type ConnectionModel, type ModelMetadata } from '$lib/providers';
	import type { MediaInput } from '$lib/types';
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

	const MEDIA_OPTIONS: { value: MediaInput; label: () => string }[] = [
		{ value: 'image', label: m.llm_media_image },
		{ value: 'pdf', label: m.llm_media_pdf },
		{ value: 'audio', label: m.llm_media_audio },
		{ value: 'video', label: m.llm_media_video },
	];
	const mediaList = (kinds: MediaInput[]) =>
		kinds.length === 0 ? m.llm_media_text_only() : MEDIA_OPTIONS.filter((o) => kinds.includes(o.value)).map((o) => o.label()).join(', ');
	const filesModel = $derived(data.models.find((model) => model.is_files_model) ?? null);
	const codingModel = $derived(data.models.find((model) => model.is_coding_model) ?? null);

	let sheetOpen = $state(false);
	let editingId = $state<string | null>(null);
	const editingModel = $derived(editingId ? (data.models.find((model) => model.id === editingId) ?? null) : null);
	// Models set up with their own key, before shared providers, keep the old fields.
	const legacy = $derived(editingModel !== null && !editingModel.connection_id);
	let connectionId = $state('');
	const connectionOptions = $derived(data.connections.map((c) => ({ value: c.id, label: connectionName(c, data.providers) })));

	let label = $state('');
	let provider = $state('anthropic');
	let apiKey = $state('');
	let baseUrl = $state('');
	let modelId = $state('');
	let inputPrice = $state('');
	let outputPrice = $state('');
	let cachedPrice = $state('');
	let mediaMode = $state<'auto' | 'listed'>('auto');
	let media = $state<MediaInput[]>([]);

	let fetching = $state(false);
	let fetchedModels = $state<{ id: string; label: string | null }[]>([]);
	let modelEntryMode = $state<'select' | 'manual'>('manual');
	let fetchError = $state<string | null>(null);

	function resetSheetState() {
		fetchedModels = [];
		modelEntryMode = 'manual';
		fetchError = null;
		fetching = false;
		prefilledFrom = null;
	}

	// A list fetched from one connection means nothing for another.
	$effect(() => {
		void connectionId;
		untrack(() => {
			fetchedModels = [];
			modelEntryMode = 'manual';
			fetchError = null;
		});
	});

	// Picking a model the catalog knows fills in its prices and what it opens (once per pick, so
	// opening a saved model never overwrites what an admin set).
	const catalogCache = new Map<string, Promise<ModelMetadata[]>>();
	function catalogFor(providerId: string): Promise<ModelMetadata[]> {
		let pending = catalogCache.get(providerId);
		if (!pending) {
			const body = new FormData();
			body.set('provider_id', providerId);
			pending = postAction<{ catalogModels: ModelMetadata[] }>('catalogModels', body).then((outcome) => {
				if (!outcome.ok) catalogCache.delete(providerId);
				return outcome.ok ? outcome.data.catalogModels : [];
			});
			catalogCache.set(providerId, pending);
		}
		return pending;
	}

	let prefilledFor = '';
	let prefilledFrom = $state<string | null>(null);
	$effect(() => {
		const key = `${connectionId}\n${modelId.trim()}`;
		if (!sheetOpen || legacy || key === prefilledFor) return;
		prefilledFor = key;
		void prefill(connectionId, modelId.trim(), key);
	});

	async function prefill(forConnection: string, forModel: string, key: string) {
		const connection = data.connections.find((c) => c.id === forConnection);
		const provider = connection ? data.providers.find((p) => p.id === connection.provider_id && p.category !== 'custom') : undefined;
		if (!provider || !forModel) return;
		const meta = (await catalogFor(provider.id)).find((model) => model.id === forModel);
		if (!meta || prefilledFor !== key) return;
		({ inputPrice, outputPrice, cachedPrice, media } = prefillFromCatalog(meta));
		mediaMode = 'listed';
		if (!label.trim()) label = meta.name;
		prefilledFrom = provider.name;
	}

	function openCreate() {
		editingId = null;
		connectionId = data.connections[0]?.id ?? '';
		prefilledFor = `${connectionId}\n`;
		label = '';
		provider = 'anthropic';
		apiKey = '';
		baseUrl = '';
		modelId = '';
		inputPrice = '';
		outputPrice = '';
		cachedPrice = '';
		mediaMode = 'auto';
		media = [];
		resetSheetState();
		sheetOpen = true;
	}

	function openEdit(model: PageData['models'][number]) {
		editingId = model.id;
		connectionId = model.connection_id ?? '';
		prefilledFor = `${connectionId}\n${model.model_id}`;
		label = model.label;
		provider = model.provider;
		apiKey = '';
		baseUrl = model.base_url ?? '';
		modelId = model.model_id;
		inputPrice = model.input_usd_per_mtok?.toString() ?? '';
		outputPrice = model.output_usd_per_mtok?.toString() ?? '';
		cachedPrice = model.cached_input_usd_per_mtok?.toString() ?? '';
		mediaMode = model.media_inputs ? 'listed' : 'auto';
		media = model.media_inputs ?? model.media_support;
		resetSheetState();
		sheetOpen = true;
	}

	function showFetched(models: ConnectionModel[]) {
		fetchedModels = models;
		modelEntryMode = 'select';
		if (fetchedModels.length > 0 && !fetchedModels.some((m) => m.id === modelId)) {
			modelId = fetchedModels[0].id;
		}
		if (fetchedModels.length === 0) {
			fetchError = m.key_no_models();
			modelEntryMode = 'manual';
		}
	}

	async function fetchModels() {
		if (!legacy) return fetchConnectionModels();
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
			showFetched(result.data.models as ConnectionModel[]);
		} else {
			fetchedModels = [];
			modelEntryMode = 'manual';
			fetchError =
				(result.type === 'failure' && (result.data?.error as string)) ||
				m.err_fetch_models();
		}
	}

	// The connection's live model list, asked of the provider with the key stored on it.
	async function fetchConnectionModels() {
		if (!connectionId) return;
		fetching = true;
		fetchError = null;
		const body = new FormData();
		body.set('connection_id', connectionId);
		const outcome = await postAction<{ connectionModels: ConnectionModel[] }>('connectionModels', body);
		fetching = false;
		if (outcome.ok) {
			showFetched(outcome.data.connectionModels);
		} else {
			fetchedModels = [];
			modelEntryMode = 'manual';
			fetchError = connectionErrorMessage(outcome.error);
		}
	}
</script>

<PageHeader title={m.admin_models()} lede={m.llm_lede()} agent="coding">
	{#snippet actions()}
		<Button type="button" variant="filled" onclick={openCreate}><IconPlus size={18} /> {m.llm_add()}</Button>
	{/snippet}
</PageHeader>

{#if data.models.length > 0}
	<section class="files-model" aria-labelledby="files-model-title">
		<div class="files-model__text">
			<h2 id="files-model-title" class="files-model__title">{m.llm_files_title()}</h2>
			<p class="files-model__lede">
				{#if filesModel}
					{m.llm_files_current({ label: filesModel.label, kinds: mediaList(filesModel.media_support) })}
				{:else}
					{m.llm_files_none()}
				{/if}
			</p>
		</div>
		{#if filesModel}
			<form method="POST" action="?/setFilesModel" use:enhance>
				<Button type="submit" variant="text" size="xs">{m.llm_files_stop()}</Button>
			</form>
		{/if}
	</section>
	<section class="files-model files-model--coding" aria-labelledby="coding-model-title">
		<div class="files-model__text">
			<h2 id="coding-model-title" class="files-model__title">{m.llm_coding_title()}</h2>
			<p class="files-model__lede">
				{#if codingModel}
					{m.llm_coding_current({ label: codingModel.label })}
				{:else}
					{m.llm_coding_none()}
				{/if}
			</p>
		</div>
		{#if codingModel}
			<form method="POST" action="?/setCodingModel" use:enhance>
				<Button type="submit" variant="text" size="xs">{m.llm_coding_stop()}</Button>
			</form>
		{/if}
	</section>
{/if}

{#if data.models.length === 0}
	<div class="empty">
		<span class="model__icon"><IconChip size={26} /></span>
		<p class="empty__text"><strong>{m.llm_empty_title()}</strong> {m.llm_empty()}</p>
	</div>
{:else}
	<ul class="models">
		{#each data.models as model (model.id)}
			<li class="model" class:model--default={model.is_default} class:model--off={!model.enabled}>
				<div class="model__top">
					<span class="model__icon"><IconChip size={24} /></span>
					<div class="model__text">
						<h3 class="model__name">{model.label}</h3>
						<p class="model__provider">{model.connection_label ?? providerName(model.provider)}</p>
					</div>
				</div>
				{#if model.is_default || model.is_files_model || model.is_coding_model || !model.enabled}
					<!-- Its own row: beside the name, the chips squeezed it to a letter per line. -->
					<div class="model__chips">
						{#if !model.enabled}<span class="chip chip--off">{m.llm_switched_off()}</span>{/if}
						{#if model.is_default}<span class="chip">{m.llm_default()}</span>{/if}
						{#if model.is_files_model}<span class="chip chip--files">{m.llm_files_chip()}</span>{/if}
						{#if model.is_coding_model}<span class="chip chip--coding">{m.llm_coding_chip()}</span>{/if}
					</div>
				{/if}
				<dl class="model__facts">
					<div><dt>{m.key_model()}</dt><dd>{model.model_id}</dd></div>
					<div><dt>{m.llm_price()}</dt><dd>{#if model.input_usd_per_mtok != null || model.output_usd_per_mtok != null}<Money value={m.llm_price_value({ input: formatUsd(model.input_usd_per_mtok ?? 0), output: formatUsd(model.output_usd_per_mtok ?? 0) })} />{:else}{m.llm_price_unset()}{/if}</dd></div>
					{#if model.connection_id}
						<div><dt>{m.key_provider()}</dt><dd>{model.connection_label ?? m.llm_none()}</dd></div>
					{:else}
						<div><dt>{m.llm_key()}</dt><dd>{model.api_key_masked || m.llm_none()}</dd></div>
					{/if}
					<div><dt>{m.llm_opens()}</dt><dd>{mediaList(model.media_support)}{#if !model.media_inputs} <span class="guessed">{m.llm_media_guessed()}</span>{/if}</dd></div>
				</dl>
				<div class="model__actions">
					<Button type="button" variant="tonal" size="xs" onclick={() => openEdit(model)}>{m.dyn_edit()}</Button>
					{#if !model.is_files_model}
						<form method="POST" action="?/setFilesModel" use:enhance>
							<input type="hidden" name="id" value={model.id} />
							<Button type="submit" variant="text" size="xs">{m.llm_files_use()}</Button>
						</form>
					{/if}
					{#if !model.is_coding_model}
						<form method="POST" action="?/setCodingModel" use:enhance>
							<input type="hidden" name="id" value={model.id} />
							<Button type="submit" variant="text" size="xs">{m.llm_coding_use()}</Button>
						</form>
					{/if}
					{#if !model.is_default}
						<form method="POST" action="?/setEnabled" use:enhance>
							<input type="hidden" name="id" value={model.id} />
							<input type="hidden" name="enabled" value={model.enabled ? 'false' : 'true'} />
							<Button type="submit" variant="text" size="xs">{model.enabled ? m.llm_switch_off() : m.llm_switch_on()}</Button>
						</form>
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

	{#if !editingId && data.connections.length === 0}
		<p class="no-connections">
			{m.prov_admin_models_none()}
			<a href="/admin/settings/providers" class="inline-link">{m.prov_go_providers()}</a>
		</p>
	{:else}
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
			{#if legacy}
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
			{:else}
				{#if editingModel}
					<input type="hidden" name="connection_id" value={connectionId} />
					<p class="runs-on">{m.prov_runs_on({ name: editingModel.connection_label ?? connectionOptions.find((o) => o.value === connectionId)?.label ?? '' })}</p>
				{:else}
					<Select label={m.key_provider()} name="connection_id" bind:value={connectionId} options={connectionOptions} />
				{/if}
				<TextField id="label" name="label" label={m.key_label()} bind:value={label} required />
			{/if}

			<Button
				type="button"
				variant="outlined"
				class="w-fit"
				disabled={fetching || (legacy ? provider !== 'fake' && !apiKey && !editingId : !connectionId)}
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
					<TextField id="cached_input_usd_per_mtok" name="cached_input_usd_per_mtok" label={m.llm_price_cached()} type="number" inputmode="decimal" min="0" step="0.0001" bind:value={cachedPrice} />
				</div>
				<p class="prices__hint">{m.llm_price_cached_hint()}</p>
				{#if prefilledFrom}
					<p class="prices__hint" role="status">{m.prov_prefilled({ name: prefilledFrom })}</p>
				{/if}
			</fieldset>
			<fieldset class="prices">
				<legend class="prices__legend">{m.llm_opens_title()}</legend>
				<p class="prices__hint">{m.llm_opens_hint()}</p>
				<label class="choice"><input type="radio" name="media_mode" value="auto" bind:group={mediaMode} /> {m.llm_media_auto()}</label>
				<label class="choice"><input type="radio" name="media_mode" value="listed" bind:group={mediaMode} /> {m.llm_media_listed()}</label>
				{#if mediaMode === 'listed'}
					<div class="media-options">
						{#each MEDIA_OPTIONS as option (option.value)}
							<label class="choice"><input type="checkbox" name="media" value={option.value} bind:group={media} /> {option.label()}</label>
						{/each}
					</div>
				{/if}
			</fieldset>
			<div class="flex gap-2 pt-2">
				<Button type="submit" variant="filled">{editingId ? m.common_save() : m.llm_add()}</Button>
				<Button type="button" variant="outlined" onclick={() => (sheetOpen = false)}>{m.common_cancel()}</Button>
			</div>
		</form>
	{/if}
</BottomSheet>

<style>
	.model--off {
		opacity: 0.65;
	}
	.chip--off {
		background: var(--md-sys-color-surface-container-highest);
		color: var(--md-sys-color-on-surface-variant);
	}
	.files-model {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		flex-wrap: wrap;
		margin-bottom: 14px;
		padding: 12px 16px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-tertiary-container);
		color: var(--md-sys-color-on-tertiary-container);
	}
	.files-model__title {
		margin: 0;
		font-family: var(--md-sys-typescale-title-medium-font);
		font-size: var(--md-sys-typescale-title-medium-size);
		font-weight: 700;
	}
	.files-model__lede {
		margin: 2px 0 0;
		font-size: 0.875rem;
	}
	.files-model--coding {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.chip.chip--coding {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.chip.chip--files {
		background: var(--md-sys-color-tertiary-container);
		color: var(--md-sys-color-on-tertiary-container);
	}
	.guessed {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
	.choice {
		display: flex;
		align-items: center;
		gap: 8px;
		min-height: 32px;
		font-size: 0.9375rem;
	}
	.choice input {
		width: 18px;
		height: 18px;
		accent-color: var(--md-sys-color-primary);
	}
	.media-options {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: 0 12px;
		padding-left: 26px;
	}
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
		gap: 14px;
		padding: 16px 20px;
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
		gap: 12px;
		padding: 16px;
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
		gap: 12px;
	}
	.model__chips {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
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
		font-size: 1.125rem;
		font-weight: 700;
		overflow-wrap: break-word;
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
		padding: 12px 12px;
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
	.no-connections {
		margin: 16px 0 8px;
		color: var(--md-sys-color-on-surface-variant);
	}
	.inline-link {
		border-radius: 4px;
		color: var(--md-sys-color-primary);
		font-weight: 600;
		text-decoration: underline;
		text-underline-offset: 2px;
	}
	.inline-link:hover {
		text-decoration-thickness: 2px;
	}
	.inline-link:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.runs-on {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.sheet-title {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.3125rem;
		font-weight: 700;
		color: var(--md-sys-color-on-surface);
	}
</style>
