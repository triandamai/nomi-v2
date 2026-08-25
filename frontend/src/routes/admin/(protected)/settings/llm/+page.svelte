<script lang="ts">
	import { enhance } from '$app/forms';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();
	let showCreateForm = $state(false);
	let editingId = $state<string | null>(null);
</script>

<h1 class="text-2xl font-semibold">LLM models</h1>

{#if form?.error}
	<p class="mt-2 text-sm text-red-600">{form.error}</p>
{/if}

<div class="mt-6 space-y-3">
	{#each data.models as model (model.id)}
		<div class="rounded-2xl border border-neutral-200 bg-white p-4">
			{#if editingId === model.id}
				<form
					method="POST"
					action="?/update"
					use:enhance={() => {
						return async ({ update }) => {
							await update();
							editingId = null;
						};
					}}
					class="space-y-2"
				>
					<input type="hidden" name="id" value={model.id} />
					<input name="label" type="text" value={model.label} required class="w-full rounded border border-neutral-300 px-2 py-1 text-sm" />
					<select name="provider" class="w-full rounded border border-neutral-300 px-2 py-1 text-sm">
						<option value="anthropic" selected={model.provider === 'anthropic'}>Anthropic</option>
						<option value="openai" selected={model.provider === 'openai'}>OpenAI</option>
						<option value="gemini" selected={model.provider === 'gemini'}>Gemini</option>
						<option value="fake" selected={model.provider === 'fake'}>Fake (testing)</option>
					</select>
					<input name="model_id" type="text" value={model.model_id} class="w-full rounded border border-neutral-300 px-2 py-1 text-sm" />
					<input name="base_url" type="text" value={model.base_url ?? ''} placeholder="Base URL (optional)" class="w-full rounded border border-neutral-300 px-2 py-1 text-sm" />
					<input name="api_key" type="password" placeholder={`Leave blank to keep ${model.api_key_masked}`} class="w-full rounded border border-neutral-300 px-2 py-1 text-sm" />
					<div class="flex gap-2">
						<button type="submit" class="rounded bg-neutral-900 px-3 py-1 text-sm text-white">Save</button>
						<button type="button" onclick={() => (editingId = null)} class="rounded border border-neutral-300 px-3 py-1 text-sm">Cancel</button>
					</div>
				</form>
			{:else}
				<div class="flex items-center justify-between">
					<div>
						<p class="font-medium">
							{model.label}
							{#if model.is_default}
								<span class="ml-2 rounded-full bg-neutral-900 px-2 py-0.5 text-xs text-white">Default</span>
							{/if}
						</p>
						<p class="text-sm text-neutral-500">{model.provider} · {model.model_id} · {model.api_key_masked}</p>
					</div>
					<div class="flex gap-2">
						<button type="button" onclick={() => (editingId = model.id)} class="text-sm text-neutral-600 hover:underline">Edit</button>
						{#if !model.is_default}
							<form method="POST" action="?/setDefault" use:enhance>
								<input type="hidden" name="id" value={model.id} />
								<button type="submit" class="text-sm text-neutral-600 hover:underline">Set default</button>
							</form>
							<form method="POST" action="?/delete" use:enhance>
								<input type="hidden" name="id" value={model.id} />
								<button type="submit" class="text-sm text-red-600 hover:underline">Delete</button>
							</form>
						{/if}
					</div>
				</div>
			{/if}
		</div>
	{/each}
</div>

<div class="mt-6">
	{#if showCreateForm}
		<form
			method="POST"
			action="?/create"
			use:enhance={() => {
				return async ({ update }) => {
					await update({ reset: true });
					showCreateForm = false;
				};
			}}
			class="max-w-lg space-y-4 rounded-2xl border border-neutral-200 bg-white p-6"
		>
			<div>
				<label for="label" class="block text-sm font-medium text-neutral-700">Label</label>
				<input id="label" name="label" type="text" required class="mt-1 w-full rounded-lg border border-neutral-300 px-3 py-2" />
			</div>
			<div>
				<label for="provider" class="block text-sm font-medium text-neutral-700">Provider</label>
				<select id="provider" name="provider" class="mt-1 w-full rounded-lg border border-neutral-300 px-3 py-2">
					<option value="anthropic">Anthropic</option>
					<option value="openai">OpenAI</option>
					<option value="gemini">Gemini</option>
					<option value="fake">Fake (testing)</option>
				</select>
			</div>
			<div>
				<label for="model_id" class="block text-sm font-medium text-neutral-700">Model ID</label>
				<input id="model_id" name="model_id" type="text" class="mt-1 w-full rounded-lg border border-neutral-300 px-3 py-2" />
			</div>
			<div>
				<label for="base_url" class="block text-sm font-medium text-neutral-700">Base URL (optional)</label>
				<input id="base_url" name="base_url" type="text" class="mt-1 w-full rounded-lg border border-neutral-300 px-3 py-2" />
			</div>
			<div>
				<label for="api_key" class="block text-sm font-medium text-neutral-700">API key</label>
				<input id="api_key" name="api_key" type="password" class="mt-1 w-full rounded-lg border border-neutral-300 px-3 py-2" />
			</div>
			<button type="submit" class="w-full rounded-lg bg-neutral-900 px-4 py-2 font-medium text-white hover:bg-neutral-800">
				Add model
			</button>
		</form>
	{:else}
		<button
			onclick={() => (showCreateForm = true)}
			class="rounded-lg border border-neutral-300 px-4 py-2 text-sm font-medium hover:bg-neutral-50"
		>
			+ Add model
		</button>
	{/if}
</div>
