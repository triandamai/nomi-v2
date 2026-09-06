<script lang="ts">
	import { enhance } from '$app/forms';
	import Button from '$lib/components/m3/Button.svelte';
	import Card from '$lib/components/m3/Card.svelte';
	import ModelKeySheet from '$lib/components/ModelKeySheet.svelte';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	let sheetOpen = $state(false);

	const currentLabel = $derived.by(() => {
		const selection = data.models.selection;
		if (!selection) return 'Default model';
		if (selection.kind === 'admin') {
			const match = data.models.admin_models.find((m) => m.id === selection.admin_model_id);
			return match?.label ?? 'Default model';
		}
		return selection.label;
	});
</script>

<div class="h-full overflow-y-auto p-8">
	<div class="max-w-lg">
		<h1 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">Model</h1>
		<p class="md-body-medium mt-1" style="color: var(--md-sys-color-on-surface-variant)">
			Choose which model Nomi uses to reply to you, or bring your own API key.
		</p>

		{#if form?.error}
			<p class="md-body-medium mt-2" style="color: var(--md-sys-color-error)">{form.error}</p>
		{/if}

		<section class="mt-6">
			<h2 class="md-title-medium" style="color: var(--md-sys-color-on-surface)">Current model</h2>
			<p class="md-body-medium mt-1" style="color: var(--md-sys-color-on-surface-variant)">{currentLabel}</p>
		</section>

		<section class="mt-8">
			<h2 class="md-title-medium" style="color: var(--md-sys-color-on-surface)">Available models</h2>
			<div class="mt-3 space-y-3">
				{#each data.models.admin_models as model (model.id)}
					{@const selected = data.models.selection?.kind === 'admin' && data.models.selection.admin_model_id === model.id}
					<Card variant="outlined" class="p-4">
						<div class="flex items-center justify-between">
							<div>
								<p class="md-title-medium" style="color: var(--md-sys-color-on-surface)">{model.label}</p>
								<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">
									{model.provider} · {model.model_id}
								</p>
							</div>
							{#if selected}
								<span class="md-label-large" style="color: var(--md-sys-color-primary)">Selected</span>
							{:else}
								<form method="POST" action="?/selectAdminModel" use:enhance>
									<input type="hidden" name="admin_model_id" value={model.id} />
									<Button type="submit" variant="outlined">Use this model</Button>
								</form>
							{/if}
						</div>
					</Card>
				{/each}
				{#if data.models.admin_models.length === 0}
					<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">
						No models have been configured yet — ask an admin to add one, or bring your own key below.
					</p>
				{/if}
			</div>
		</section>

		<section class="mt-8">
			<h2 class="md-title-medium" style="color: var(--md-sys-color-on-surface)">Your own API key</h2>
			{#if data.models.selection?.kind === 'custom'}
				<p class="md-body-medium mt-1" style="color: var(--md-sys-color-on-surface-variant)">
					Using {data.models.selection.label} ({data.models.selection.provider} · {data.models.selection.api_key_masked})
				</p>
			{:else}
				<p class="md-body-medium mt-1" style="color: var(--md-sys-color-on-surface-variant)">
					Bring your own API key to use a model outside the list above.
				</p>
			{/if}
			<div class="mt-3">
				<Button type="button" variant="outlined" onclick={() => (sheetOpen = true)}>
					{data.models.selection?.kind === 'custom' ? 'Change your key' : '+ Add your own key'}
				</Button>
			</div>
		</section>
	</div>
</div>

<ModelKeySheet bind:open={sheetOpen} error={form?.error ?? null} />
