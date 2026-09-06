<script lang="ts">
	import { deserialize, enhance } from '$app/forms';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import Select from '$lib/components/m3/Select.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';

	const PROVIDER_OPTIONS = [
		{ value: 'anthropic', label: 'Anthropic' },
		{ value: 'openai', label: 'OpenAI' },
		{ value: 'gemini', label: 'Gemini' },
		{ value: 'fake', label: 'Fake (testing)' },
	];

	let {
		open = $bindable(false),
		error = null,
		onSaved,
	}: {
		open?: boolean;
		error?: string | null;
		onSaved?: () => void;
	} = $props();

	let label = $state('');
	let provider = $state('anthropic');
	let apiKey = $state('');
	let baseUrl = $state('');
	let modelId = $state('');

	let fetching = $state(false);
	let fetchedModels = $state<{ id: string; label: string | null }[]>([]);
	let modelEntryMode = $state<'select' | 'manual'>('manual');
	let fetchError = $state<string | null>(null);

	// Every open starts from a clean slate — there's only ever one active custom selection, so
	// there's no "editing" state to pre-fill, and re-entering the key each time is an accepted
	// simplification (see the /models + chat-page design discussion).
	$effect(() => {
		if (open) {
			label = '';
			provider = 'anthropic';
			apiKey = '';
			baseUrl = '';
			modelId = '';
			fetchedModels = [];
			modelEntryMode = 'manual';
			fetchError = null;
			fetching = false;
		}
	});

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

<BottomSheet bind:open>
	<h2 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">Use your own API key</h2>

	{#if error}
		<p class="md-body-medium mt-2" style="color: var(--md-sys-color-error)">{error}</p>
	{/if}

	<form
		method="POST"
		action="?/selectCustomModel"
		use:enhance={() => {
			return async ({ update }) => {
				await update({ reset: true });
				open = false;
				onSaved?.();
			};
		}}
		class="mt-4 flex flex-col gap-3"
	>
		<TextField id="label" name="label" label="Label" bind:value={label} required />
		<Select label="Provider" name="provider" bind:value={provider} options={PROVIDER_OPTIONS} />
		<TextField id="api_key" name="api_key" type="password" label="API key" bind:value={apiKey} />
		<TextField id="base_url" name="base_url" label="Base URL (optional)" bind:value={baseUrl} />

		<Button
			type="button"
			variant="outlined"
			class="w-fit"
			disabled={fetching || (provider !== 'fake' && !apiKey)}
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
			<Button type="submit" variant="filled">Save &amp; validate</Button>
			<Button type="button" variant="outlined" onclick={() => (open = false)}>Cancel</Button>
		</div>
	</form>
</BottomSheet>
