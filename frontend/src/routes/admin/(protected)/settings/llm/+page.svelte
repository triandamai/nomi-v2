<script lang="ts">
	import { enhance } from '$app/forms';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	const settings = $derived(form?.settings ?? data.settings);
</script>

<h1 class="text-2xl font-semibold">LLM provider settings</h1>

<form method="POST" use:enhance class="mt-6 max-w-lg space-y-4 rounded-2xl border border-neutral-200 bg-white p-6">
	{#if form?.error}
		<p class="text-sm text-red-600">{form.error}</p>
	{/if}
	{#if form?.success}
		<p class="text-sm text-green-600">Settings saved.</p>
	{/if}

	<div>
		<label for="provider" class="block text-sm font-medium text-neutral-700">Provider</label>
		<select
			id="provider"
			name="provider"
			class="mt-1 w-full rounded-lg border border-neutral-300 px-3 py-2"
			value={settings?.provider ?? 'anthropic'}
		>
			<option value="anthropic">Anthropic</option>
			<option value="openai">OpenAI</option>
			<option value="gemini">Gemini</option>
			<option value="fake">Fake (testing)</option>
		</select>
	</div>
	<div>
		<label for="model_id" class="block text-sm font-medium text-neutral-700">Model ID</label>
		<input
			id="model_id"
			name="model_id"
			type="text"
			value={settings?.model_id ?? ''}
			class="mt-1 w-full rounded-lg border border-neutral-300 px-3 py-2"
		/>
	</div>
	<div>
		<label for="base_url" class="block text-sm font-medium text-neutral-700">Base URL (optional)</label>
		<input
			id="base_url"
			name="base_url"
			type="text"
			value={settings?.base_url ?? ''}
			class="mt-1 w-full rounded-lg border border-neutral-300 px-3 py-2"
		/>
	</div>
	<div>
		<label for="api_key" class="block text-sm font-medium text-neutral-700">
			API key
			{#if settings?.api_key_masked}
				<span class="text-neutral-400">(current: {settings.api_key_masked})</span>
			{/if}
		</label>
		<input
			id="api_key"
			name="api_key"
			type="password"
			placeholder={settings?.api_key_masked ? 'Leave blank to keep current key' : ''}
			class="mt-1 w-full rounded-lg border border-neutral-300 px-3 py-2"
		/>
	</div>
	<button
		type="submit"
		class="w-full rounded-lg bg-neutral-900 px-4 py-2 font-medium text-white hover:bg-neutral-800"
	>
		Save
	</button>
</form>
