<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { deserialize, enhance } from '$app/forms';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import Select from '$lib/components/m3/Select.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';

	const PROVIDER_OPTIONS = [
		{ value: 'anthropic', label: 'Anthropic' },
		{ value: 'openai', label: 'OpenAI' },
		{ value: 'openrouter', label: 'OpenRouter' },
		{ value: 'gemini', label: 'Gemini' },
		{ value: 'deepseek', label: 'DeepSeek' },
		{ value: 'fake', label: m.key_fake() },
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

<BottomSheet bind:open>
	<h2 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">{m.key_title()}</h2>

	{#if error}
		<p class="md-body-medium mt-2" style="color: var(--md-sys-color-error)">{error}</p>
	{/if}

	<form
		method="POST"
		action="?/selectCustomModel"
		use:enhance={() => {
			return async ({ result, update }) => {
				await update({ reset: result.type === 'success' });
				if (result.type === 'success') {
					open = false;
					onSaved?.();
				}
			};
		}}
		class="mt-4 flex flex-col gap-3"
	>
		<TextField id="label" name="label" label={m.key_label()} bind:value={label} required />
		<Select label={m.key_provider()} name="provider" bind:value={provider} options={PROVIDER_OPTIONS} />
		<TextField id="api_key" name="api_key" type="password" label={m.key_api_key()} bind:value={apiKey} />
		<TextField id="base_url" name="base_url" label={m.key_base_url()} bind:value={baseUrl} />

		<Button
			type="button"
			variant="outlined"
			class="w-fit"
			disabled={fetching || (provider !== 'fake' && !apiKey)}
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

		<div class="flex gap-2 pt-2">
			<Button type="submit" variant="filled">{m.key_save_validate()}</Button>
			<Button type="button" variant="outlined" onclick={() => (open = false)}>{m.common_cancel()}</Button>
		</div>
	</form>
</BottomSheet>
