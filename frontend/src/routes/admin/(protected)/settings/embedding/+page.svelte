<script lang="ts">
	import { enhance } from '$app/forms';
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
</script>

<h1 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">Embedding provider</h1>
<p class="md-body-large mt-2" style="color: var(--md-sys-color-on-surface-variant)">
	Used by the personality memory feature's semantic search. Changing the provider stops older
	memories from being retrieved until new ones are stored under the new provider — they aren't
	deleted, just no longer comparable to new queries.
</p>

{#if form?.error}
	<p class="md-body-medium mt-2" style="color: var(--md-sys-color-error)">{form.error}</p>
{/if}

<div class="mt-6 max-w-lg">
	<Card variant="outlined" class="p-6">
		<form method="POST" action="?/update" class="space-y-4" use:enhance>
			<Select
				label="Provider"
				name="provider"
				options={PROVIDER_OPTIONS}
				value={data.settings?.provider ?? 'openai'}
			/>
			<TextField id="model_id" name="model_id" label="Model ID" value={data.settings?.model_id ?? ''} />
			<TextField
				id="api_key"
				name="api_key"
				type="password"
				label="API key"
				placeholder={data.settings ? `Leave blank to keep ${data.settings.api_key_masked}` : 'Required'}
			/>
			<TextField id="base_url" name="base_url" label="Base URL (optional)" value={data.settings?.base_url ?? ''} />
			<Button type="submit" variant="filled" class="w-full">Save</Button>
		</form>
	</Card>
</div>
