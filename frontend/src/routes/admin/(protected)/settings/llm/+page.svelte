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
		{ value: 'anthropic', label: 'Anthropic' },
		{ value: 'openai', label: 'OpenAI' },
		{ value: 'gemini', label: 'Gemini' },
		{ value: 'fake', label: 'Fake (testing)' },
	];

	let sheetOpen = $state(false);
	let editingId = $state<string | null>(null);

	let label = $state('');
	let provider = $state('anthropic');
	let apiKey = $state('');
	let baseUrl = $state('');
	let modelId = $state('');

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

<h1 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">LLM models</h1>

<div class="mt-6 space-y-3">
	{#each data.models as model (model.id)}
		<Card variant="outlined" class="p-4">
			<div class="flex items-center justify-between">
				<div>
					<p class="md-title-medium" style="color: var(--md-sys-color-on-surface)">
						{model.label}
						{#if model.is_default}
							<span
								class="md-label-medium ml-2 rounded-full px-2 py-0.5"
								style="background: var(--md-sys-color-primary); color: var(--md-sys-color-on-primary)"
							>
								Default
							</span>
						{/if}
					</p>
					<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">
						{model.provider} · {model.model_id} · {model.api_key_masked}
					</p>
				</div>
				<div class="flex items-center gap-2">
					<Button type="button" variant="text" onclick={() => openEdit(model)}>Edit</Button>
					{#if !model.is_default}
						<form method="POST" action="?/setDefault" use:enhance>
							<input type="hidden" name="id" value={model.id} />
							<Button type="submit" variant="text">Set default</Button>
						</form>
						<form method="POST" action="?/delete" use:enhance>
							<input type="hidden" name="id" value={model.id} />
							<Button type="submit" variant="text" style="color: var(--md-sys-color-error)">Delete</Button>
						</form>
					{/if}
				</div>
			</div>
		</Card>
	{/each}
</div>

<div class="mt-6">
	<Button type="button" variant="outlined" onclick={openCreate}>+ Add model</Button>
</div>

<BottomSheet bind:open={sheetOpen}>
	<h2 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">
		{editingId ? 'Edit model' : 'Add model'}
	</h2>

	{#if form?.error}
		<p class="md-body-medium mt-2" style="color: var(--md-sys-color-error)">{form.error}</p>
	{/if}

	<form
		method="POST"
		action={editingId ? '?/update' : '?/create'}
		use:enhance={() => {
			return async ({ update }) => {
				await update({ reset: false });
				sheetOpen = false;
			};
		}}
		class="mt-4 flex flex-col gap-3"
	>
		{#if editingId}
			<input type="hidden" name="id" value={editingId} />
		{/if}
		<TextField id="label" name="label" label="Label" bind:value={label} required />
		<Select label="Provider" name="provider" bind:value={provider} options={PROVIDER_OPTIONS} />
		<TextField
			id="api_key"
			name="api_key"
			type="password"
			label="API key"
			bind:value={apiKey}
			placeholder={editingId ? 'Leave blank to keep the existing key' : undefined}
		/>
		<TextField id="base_url" name="base_url" label="Base URL (optional)" bind:value={baseUrl} />

		<Button
			type="button"
			variant="outlined"
			class="w-fit"
			disabled={fetching || (provider !== 'fake' && !apiKey && !editingId)}
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
			<TextField id="model_id" name="model_id" label="Model ID" bind:value={modelId} required />
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
			<Button type="submit" variant="filled">{editingId ? 'Save' : 'Add model'}</Button>
			<Button type="button" variant="outlined" onclick={() => (sheetOpen = false)}>Cancel</Button>
		</div>
	</form>
</BottomSheet>
