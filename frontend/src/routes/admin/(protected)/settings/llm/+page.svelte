<script lang="ts">
	import { enhance } from '$app/forms';
	import Button from '$lib/components/m3/Button.svelte';
	import Card from '$lib/components/m3/Card.svelte';
	import Select from '$lib/components/m3/Select.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();
	let showCreateForm = $state(false);
	let editingId = $state<string | null>(null);

	const PROVIDER_OPTIONS = [
		{ value: 'anthropic', label: 'Anthropic' },
		{ value: 'openai', label: 'OpenAI' },
		{ value: 'gemini', label: 'Gemini' },
		{ value: 'fake', label: 'Fake (testing)' },
	];
</script>

<h1 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">LLM models</h1>

{#if form?.error}
	<p class="md-body-medium mt-2" style="color: var(--md-sys-color-error)">{form.error}</p>
{/if}

<div class="mt-6 space-y-3">
	{#each data.models as model (model.id)}
		<Card variant="outlined" class="p-4">
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
					<TextField id="label" name="label" label="Label" value={model.label} required />
					<Select label="Provider" name="provider" options={PROVIDER_OPTIONS} value={model.provider} />
					<TextField id="model_id" name="model_id" label="Model ID" value={model.model_id} />
					<TextField id="base_url" name="base_url" label="Base URL (optional)" value={model.base_url ?? ''} />
					<TextField
						id="api_key"
						name="api_key"
						type="password"
						label="API key"
						placeholder={`Leave blank to keep ${model.api_key_masked}`}
					/>
					<div class="flex gap-2 pt-2">
						<Button type="submit" variant="filled">Save</Button>
						<Button type="button" variant="outlined" onclick={() => (editingId = null)}>Cancel</Button>
					</div>
				</form>
			{:else}
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
						<Button type="button" variant="text" onclick={() => (editingId = model.id)}>Edit</Button>
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
			{/if}
		</Card>
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
			class="max-w-lg space-y-4 p-6"
			style="background: var(--md-sys-color-surface-container-low); border-radius: var(--md-sys-shape-corner-large)"
		>
			<TextField id="label" name="label" label="Label" required />
			<Select label="Provider" name="provider" options={PROVIDER_OPTIONS} />
			<TextField id="model_id" name="model_id" label="Model ID" />
			<TextField id="base_url" name="base_url" label="Base URL (optional)" />
			<TextField id="api_key" name="api_key" type="password" label="API key" />
			<Button type="submit" variant="filled" class="w-full">Add model</Button>
		</form>
	{:else}
		<Button type="button" variant="outlined" onclick={() => (showCreateForm = true)}>+ Add model</Button>
	{/if}
</div>
