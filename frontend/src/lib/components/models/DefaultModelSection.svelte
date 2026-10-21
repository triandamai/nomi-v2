<script lang="ts">
	// The Model page's default model: what new chats start on (each chat can switch in its
	// composer). The same picker as a chat's composer, offering what's switched on in Manage
	// models, and Manage models itself. A pick shows straight away and is put back if it can't
	// be saved (`setDefaultModel`); `options` is null when the models couldn't be read.
	import type { Snippet } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import Button from '$lib/components/m3/Button.svelte';
	import IconSliders from '$lib/components/icons/IconSliders.svelte';
	import ModelManager from './ModelManager.svelte';
	import ModelSelector from './ModelSelector.svelte';
	import { applyChanges, choiceFor, type ModelChange, type ModelChoice, type ModelOptions } from '$lib/modelPicker';
	import { postAction } from '$lib/providerActions';

	let {
		options,
		onchange,
		extra,
	}: {
		options: ModelOptions | null;
		/** More of the page's buttons, after Manage models. */
		extra?: Snippet;
		/** The default as shown (ahead of the server while a pick is saving). */
		onchange?: (choice: ModelChoice | null) => void;
	} = $props();

	const uid = $props.id();

	let groups = $derived(options?.groups ?? []);
	let current = $derived(options?.current ?? null);
	let manageOpen = $state(false);
	let error = $state<string | null>(null);

	function show(choice: ModelChoice | null) {
		current = choice;
		onchange?.(choice);
	}

	async function pick(key: string) {
		const previous = current;
		const guess = choiceFor(groups, key);
		error = null;
		show(guess ? { ...guess, chat_only: false } : previous);
		const body = new FormData();
		body.set('key', key);
		const outcome = await postAction<{ defaultModel: ModelChoice }>('setDefaultModel', body);
		if (outcome.ok) {
			show(outcome.data.defaultModel);
		} else {
			show(previous);
			error = m.models_default_failed();
		}
	}

	const saved = (changes: ModelChange[]) => (groups = applyChanges(groups, changes));
</script>

<section class="mt-6" aria-labelledby="{uid}-title">
	<h2 id="{uid}-title" class="md-title-medium" style="color: var(--md-sys-color-on-surface)">{m.models_default()}</h2>
	<p class="md-body-medium mt-1" style="color: var(--md-sys-color-on-surface-variant)">{m.models_default_hint()}</p>
	{#if options}
		<div class="default-row mt-3">
			<ModelSelector {groups} selected={current} onselect={pick} onmanage={() => (manageOpen = true)} />
			<Button type="button" variant="text" onclick={() => (manageOpen = true)}>
				<IconSliders size={18} />
				{m.msel_manage()}
			</Button>
			{@render extra?.()}
		</div>
		{#if error}
			<p class="md-body-small mt-2" role="alert" style="color: var(--md-sys-color-error)">{error}</p>
		{/if}
	{:else}
		<p class="md-body-medium mt-3" style="color: var(--md-sys-color-error)">{m.models_load_failed()}</p>
		{#if extra}<div class="default-row mt-2">{@render extra()}</div>{/if}
	{/if}
</section>

{#if options}
	<ModelManager bind:open={manageOpen} {groups} onsaved={saved} />
{/if}

<style>
	.default-row {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px;
		min-width: 0;
	}
	@media (pointer: coarse) {
		.default-row :global(.m3-button) {
			height: 44px;
		}
	}
</style>
