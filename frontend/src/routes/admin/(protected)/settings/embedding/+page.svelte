<script lang="ts">
	import { deserialize } from '$app/forms';
	import { enhance } from '$app/forms';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import Card from '$lib/components/m3/Card.svelte';
	import Select from '$lib/components/m3/Select.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	const PROVIDER_OPTIONS = [
		{ value: 'openai', label: 'OpenAI' },
		{ value: 'gemini', label: 'Gemini' },
		{ value: 'cohere', label: 'Cohere' },
		{ value: 'fake', label: 'Fake (testing)' },
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
				fetchError = 'This provider returned no models — enter the model ID manually.';
				modelEntryMode = 'manual';
			}
		} else {
			fetchedModels = [];
			modelEntryMode = 'manual';
			fetchError =
				(result.type === 'failure' && (result.data?.error as string)) ||
				'Could not fetch models — enter the model ID manually.';
		}
	}
</script>

<h1 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">Embedding provider</h1>
<p class="md-body-large mt-2" style="color: var(--md-sys-color-on-surface-variant)">
	Used by the personality memory feature's semantic search. Changing the provider stops older
	memories from being retrieved until new ones are stored under the new provider — they aren't
	deleted, just no longer comparable to new queries.
</p>

<div class="mt-6 max-w-lg">
	<Card variant="outlined" class="p-6">
		{#if data.settings}
			<p class="md-title-medium" style="color: var(--md-sys-color-on-surface)">{data.settings.provider}</p>
			<p class="md-body-medium mt-1" style="color: var(--md-sys-color-on-surface-variant)">
				{data.settings.model_id} · {data.settings.api_key_masked}
			</p>
		{:else}
			<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">No embedding provider configured yet.</p>
		{/if}
		<Button type="button" variant="outlined" class="mt-4" onclick={openEdit}>
			{data.settings ? 'Edit' : 'Configure'}
		</Button>
	</Card>
</div>

<BottomSheet bind:open={sheetOpen}>
	<h2 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">Embedding provider</h2>

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
		<Select label="Provider" name="provider" bind:value={provider} options={PROVIDER_OPTIONS} />
		<TextField
			id="api_key"
			name="api_key"
			type="password"
			label="API key"
			bind:value={apiKey}
			placeholder={data.settings ? `Leave blank to keep ${data.settings.api_key_masked}` : 'Required'}
		/>
		<TextField id="base_url" name="base_url" label="Base URL (optional)" bind:value={baseUrl} />

		<Button
			type="button"
			variant="outlined"
			class="w-fit"
			disabled={fetching || (provider !== 'fake' && !apiKey && !data.settings)}
			onclick={fetchModels}
		>
			{fetching ? 'Fetching…' : 'Fetch models'}
		</Button>
		{#if fetchError}
			<p class="md-body-small" style="color: var(--md-sys-color-on-surface-variant)">{fetchError}</p>
		{/if}

		{#if modelEntryMode === 'select' && fetchedModels.length > 0}
			<Select
				label="Model"
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
				Enter model ID manually instead
			</button>
		{:else}
			<TextField id="model_id" name="model_id" label="Model ID" bind:value={modelId} />
			{#if fetchedModels.length > 0}
				<button
					type="button"
					class="md-label-large self-start"
					style="color: var(--md-sys-color-primary); background: none; border: none; cursor: pointer; padding: 0"
					onclick={() => (modelEntryMode = 'select')}
				>
					Choose from fetched list instead
				</button>
			{/if}
		{/if}

		<div class="flex gap-2 pt-2">
			<Button type="submit" variant="filled">Save</Button>
			<Button type="button" variant="outlined" onclick={() => (sheetOpen = false)}>Cancel</Button>
		</div>
	</form>
</BottomSheet>
