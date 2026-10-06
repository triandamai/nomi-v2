<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { deserialize } from '$app/forms';
	import { enhance } from '$app/forms';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import IconMemory from '$lib/components/icons/IconMemory.svelte';
	import Select from '$lib/components/m3/Select.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	const PROVIDER_OPTIONS = [
		{ value: 'openai', label: 'OpenAI' },
		{ value: 'gemini', label: 'Gemini' },
		{ value: 'cohere', label: 'Cohere' },
		{ value: 'fake', label: m.key_fake() },
	];

	let sheetOpen = $state(false);
	let provider = $state(data.settings?.provider ?? 'openai');
	let apiKey = $state('');
	let baseUrl = $state(data.settings?.base_url ?? '');
	let modelId = $state(data.settings?.model_id ?? '');

	let fetching = $state(false);
	let fetchedModels = $state<{ id: string; label: string | null }[]>([]);
	let modelEntryMode = $state<'select' | 'manual'>('manual');
	let fetchError = $state<string | null>(null);

	function openEdit() {
		provider = data.settings?.provider ?? 'openai';
		apiKey = '';
		baseUrl = data.settings?.base_url ?? '';
		modelId = data.settings?.model_id ?? '';
		fetchedModels = [];
		modelEntryMode = 'manual';
		fetchError = null;
		fetching = false;
		sheetOpen = true;
	}

	async function fetchModels() {
		fetching = true;
		fetchError = null;

		const body = new FormData();
		body.set('provider', provider);
		body.set('api_key', apiKey);
		body.set('base_url', baseUrl);

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

<PageHeader
	title={m.admin_embeddings()}
	lede={m.emb_lede()}
	agent="memory"
/>

<section class="provider" aria-label={m.emb_current()}>
	<span class="provider__icon"><IconMemory size={26} /></span>
	<div class="provider__text">
		{#if data.settings}
			<span class="nomi-meta">{m.emb_in_use()}</span>
			<h2 class="provider__name">{PROVIDER_OPTIONS.find((o) => o.value === data.settings?.provider)?.label ?? data.settings.provider}</h2>
			<p class="provider__facts">{data.settings.model_id} · {data.settings.api_key_masked}</p>
		{:else}
			<h2 class="provider__name">{m.emb_not_set()}</h2>
			<p class="provider__facts">{m.emb_not_set_hint()}</p>
		{/if}
	</div>
	<Button type="button" variant={data.settings ? 'tonal' : 'filled'} onclick={openEdit}>{data.settings ? m.emb_change() : m.emb_set_up()}</Button>
</section>

<BottomSheet bind:open={sheetOpen}>
	<h2 class="sheet-title">{m.emb_title()}</h2>

	{#if form?.error}
		<p class="md-body-medium mt-2" style="color: var(--md-sys-color-error)">{form.error}</p>
	{/if}

	<form
		method="POST"
		action="?/update"
		use:enhance={() => {
			return async ({ update }) => {
				await update({ reset: false });
				sheetOpen = false;
			};
		}}
		class="mt-4 flex flex-col gap-3"
	>
		<Select label={m.key_provider()} name="provider" bind:value={provider} options={PROVIDER_OPTIONS} />
		<TextField
			id="api_key"
			name="api_key"
			type="password"
			label={m.key_api_key()}
			bind:value={apiKey}
			placeholder={data.settings ? m.emb_keep_key({ key: data.settings.api_key_masked }) : m.emb_required()}
		/>
		<TextField id="base_url" name="base_url" label={m.key_base_url()} bind:value={baseUrl} />

		<Button
			type="button"
			variant="outlined"
			class="w-fit"
			disabled={fetching || (provider !== 'fake' && !apiKey && !data.settings)}
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
			<TextField id="model_id" name="model_id" label={m.key_model_id()} bind:value={modelId} />
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

		<div class="flex gap-2 pt-2">
			<Button type="submit" variant="filled">{m.common_save()}</Button>
			<Button type="button" variant="outlined" onclick={() => (sheetOpen = false)}>{m.common_cancel()}</Button>
		</div>
	</form>
</BottomSheet>

<style>
	.provider {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 16px 20px;
		max-width: 720px;
		padding: 22px 24px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
	}
	.provider__icon {
		display: grid;
		flex: none;
		place-items: center;
		width: 56px;
		height: 56px;
		border-radius: 18px;
		background: var(--md-sys-color-tertiary-container);
		color: var(--md-sys-color-on-tertiary-container);
	}
	.provider__text {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 2px;
		min-width: 200px;
	}
	.provider__name {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.375rem;
		font-weight: 700;
	}
	.provider__facts {
		margin: 0;
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
		overflow-wrap: anywhere;
	}
	.sheet-title {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.5rem;
		font-weight: 700;
		color: var(--md-sys-color-on-surface);
	}
</style>
