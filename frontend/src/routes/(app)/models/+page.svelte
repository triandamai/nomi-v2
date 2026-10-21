<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { enhance } from '$app/forms';
	import Button from '$lib/components/m3/Button.svelte';
	import Select from '$lib/components/m3/Select.svelte';
	import DefaultModelSection from '$lib/components/models/DefaultModelSection.svelte';
	import type { ModelChoice } from '$lib/modelPicker';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	// The default for new chats, as shown (a pick moves it ahead of the server).
	let defaultModel = $derived<ModelChoice | null>(data.modelOptions?.current ?? null);
	const defaultLabel = $derived(defaultModel?.label || defaultModel?.model_id || m.models_default());

	// Koda's model: Nomi's coding model, the same as chats, or one of Nomi's models.
	const defaultCodingLabel = $derived(data.models.admin_models.find((model) => model.id === data.models.default_coding_model_id)?.label ?? null);
	const codingOptions = $derived([
		{ value: 'default', label: defaultCodingLabel ? m.models_coding_default({ label: defaultCodingLabel }) : m.models_coding_default_chat() },
		{ value: 'same_as_chat', label: m.models_coding_same({ label: defaultLabel }) },
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

		<DefaultModelSection options={data.modelOptions} onchange={(choice) => (defaultModel = choice)}>
			{#snippet extra()}
				<Button href="/providers" variant="text">{m.prov_manage()}</Button>
			{/snippet}
		</DefaultModelSection>

		<section class="mt-8">
			<h2 class="md-title-medium" style="color: var(--md-sys-color-on-surface)">{m.models_coding_title()}</h2>
			<p class="md-body-medium mt-1" style="color: var(--md-sys-color-on-surface-variant)">{m.models_coding_hint()}</p>
			{#if form?.error}
				<p class="md-body-medium mt-2" role="alert" style="color: var(--md-sys-color-error)">{form.error}</p>
			{/if}
			<form method="POST" action="?/selectCodingModel" use:enhance class="mt-3">
				<Select label={m.models_coding_label()} name="coding" options={codingOptions} bind:value={codingPick} />
				{#if codingPick !== codingValue}
					<div class="mt-3"><Button type="submit" variant="filled">{m.common_save()}</Button></div>
				{/if}
			</form>
		</section>
	</div>
</div>
