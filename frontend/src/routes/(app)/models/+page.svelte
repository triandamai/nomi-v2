<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { enhance } from '$app/forms';
	import Button from '$lib/components/m3/Button.svelte';
	import Card from '$lib/components/m3/Card.svelte';
	import Select from '$lib/components/m3/Select.svelte';
	import ModelKeySheet from '$lib/components/ModelKeySheet.svelte';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	let sheetOpen = $state(false);

	const currentLabel = $derived.by(() => {
		const selection = data.models.selection;
		if (!selection) return m.models_default();
		if (selection.kind === 'admin') {
			const match = data.models.admin_models.find((m) => m.id === selection.admin_model_id);
			return match?.label ?? m.models_default();
		}
		return selection.label;
	});

	// Koda's model: Nomi's coding model, the same as chats, or one of Nomi's models.
	const defaultCodingLabel = $derived(data.models.admin_models.find((model) => model.id === data.models.default_coding_model_id)?.label ?? null);
	const codingOptions = $derived([
		{ value: 'default', label: defaultCodingLabel ? m.models_coding_default({ label: defaultCodingLabel }) : m.models_coding_default_chat() },
		{ value: 'same_as_chat', label: m.models_coding_same({ label: currentLabel }) },
		...data.models.admin_models.map((model) => ({ value: model.id, label: model.label })),
	]);
	const codingValue = $derived(
		data.models.coding.kind === 'admin' ? data.models.coding.admin_model_id : data.models.coding.kind,
	);
	let codingPick = $state('default');
	$effect.pre(() => {
		codingPick = codingValue;
	});
</script>

<div class="h-full overflow-y-auto px-4 py-8 md:px-10">
	<div class="max-w-lg">
		<h1 class="md-display-small" style="color: var(--md-sys-color-on-surface)">{m.models_title()}</h1>
		<p class="md-body-medium mt-1" style="color: var(--md-sys-color-on-surface-variant)">
			{m.models_lede()}
		</p>

		{#if form?.error}
			<p class="md-body-medium mt-2" style="color: var(--md-sys-color-error)">{form.error}</p>
		{/if}

		<section class="mt-6">
			<h2 class="md-title-medium" style="color: var(--md-sys-color-on-surface)">{m.models_current()}</h2>
			<p class="md-body-medium mt-1" style="color: var(--md-sys-color-on-surface-variant)">{currentLabel}</p>
		</section>

		<section class="mt-8">
			<h2 class="md-title-medium" style="color: var(--md-sys-color-on-surface)">{m.models_available()}</h2>
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
								<span class="md-label-large" style="color: var(--md-sys-color-primary)">{m.models_selected()}</span>
							{:else}
								<form method="POST" action="?/selectAdminModel" use:enhance>
									<input type="hidden" name="admin_model_id" value={model.id} />
									<Button type="submit" variant="outlined">{m.models_use()}</Button>
								</form>
							{/if}
						</div>
					</Card>
				{/each}
				{#if data.models.admin_models.length === 0}
					<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">
						{m.models_none()}
					</p>
				{/if}
			</div>
		</section>

		<section class="mt-8">
			<h2 class="md-title-medium" style="color: var(--md-sys-color-on-surface)">{m.models_coding_title()}</h2>
			<p class="md-body-medium mt-1" style="color: var(--md-sys-color-on-surface-variant)">{m.models_coding_hint()}</p>
			<form method="POST" action="?/selectCodingModel" use:enhance class="mt-3">
				<Select label={m.models_coding_label()} name="coding" options={codingOptions} bind:value={codingPick} />
				{#if codingPick !== codingValue}
					<div class="mt-3"><Button type="submit" variant="filled">{m.common_save()}</Button></div>
				{/if}
			</form>
		</section>

		<section class="mt-8">
			<h2 class="md-title-medium" style="color: var(--md-sys-color-on-surface)">{m.models_own_key()}</h2>
			{#if data.models.selection?.kind === 'custom'}
				<p class="md-body-medium mt-1" style="color: var(--md-sys-color-on-surface-variant)">
					{m.models_using({ label: data.models.selection.label, provider: data.models.selection.provider, key: data.models.selection.api_key_masked })}
				</p>
			{:else}
				<p class="md-body-medium mt-1" style="color: var(--md-sys-color-on-surface-variant)">
					{m.models_own_key_hint()}
				</p>
			{/if}
			<div class="mt-3">
				<Button type="button" variant="outlined" onclick={() => (sheetOpen = true)}>
					{data.models.selection?.kind === 'custom' ? m.models_change_key() : m.models_add_key()}
				</Button>
			</div>
		</section>
	</div>
</div>

<ModelKeySheet bind:open={sheetOpen} error={form?.error ?? null} />
