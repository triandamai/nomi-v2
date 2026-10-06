<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { deserialize } from '$app/forms';
	import Button from '$lib/components/m3/Button.svelte';
	import Select from '$lib/components/m3/Select.svelte';
	import { kindLabel, MEMORY_KINDS } from '$lib/memory';
	import type { MemoryKind } from '$lib/types';

	// Rewrites one memory, from the Memory page or straight from a reply that used it. Saves
	// through the Memory page's edit action, so it works wherever it's shown.
	let {
		id,
		content,
		kind,
		onsaved,
		oncancel,
	}: { id: string; content: string; kind: MemoryKind; onsaved: (content: string, kind: MemoryKind) => void; oncancel: () => void } = $props();

	let text = $state('');
	let pickedKind = $state<MemoryKind>('fact');
	let saving = $state(false);
	let error = $state<string | null>(null);
	$effect.pre(() => {
		text = content;
		pickedKind = kind;
	});
	const KINDS = MEMORY_KINDS.map((value) => ({ value, label: kindLabel(value) }));

	async function save(event: SubmitEvent) {
		event.preventDefault();
		saving = true;
		error = null;
		const body = new FormData();
		body.set('id', id);
		body.set('content', text);
		body.set('kind', pickedKind);
		const result = deserialize(await (await fetch('/memory?/edit', { method: 'POST', body })).text());
		saving = false;
		if (result.type === 'success') onsaved(text.trim(), pickedKind);
		else error = (result.type === 'failure' && (result.data?.error as string)) || m.mem_edit_failed();
	}
</script>

<form class="edit" onsubmit={save}>
	<label class="edit__field">
		<span class="edit__label">{m.mem_edit_label()}</span>
		<textarea bind:value={text} rows="2" maxlength="200" required class="edit__text"></textarea>
		<span class="edit__hint">{m.mem_edit_hint()}</span>
	</label>
	<Select label={m.mem_kind()} name="kind" bind:value={pickedKind} options={KINDS} />
	{#if error}<p class="edit__error" role="alert">{error}</p>{/if}
	<div class="edit__actions">
		<Button type="button" variant="text" onclick={oncancel}>{m.common_cancel()}</Button>
		<Button type="submit" variant="filled" disabled={saving || !text.trim()}>{m.common_save()}</Button>
	</div>
</form>

<style>
	.edit {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
	.edit__field {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.edit__label {
		font-size: 0.75rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.edit__text {
		padding: 12px 14px;
		border: 1px solid var(--md-sys-color-outline);
		border-radius: var(--md-sys-shape-corner-medium);
		background: transparent;
		color: var(--md-sys-color-on-surface);
		font: inherit;
		resize: vertical;
	}
	.edit__text:focus {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: -1px;
	}
	.edit__hint {
		font-size: 0.75rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.edit__error {
		margin: 0;
		color: var(--md-sys-color-error);
		font-size: 0.875rem;
	}
	.edit__actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
	}
</style>
