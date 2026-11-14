<script lang="ts">
	import { CircleAlert } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Admin → Text area: TextField's label · field · helper for text longer than a line (a
	// notification's message, an agent's instructions, an article). Same surface, border-strong and
	// focus ring; an error swaps the border and takes the helper's place. With a limit it counts the
	// characters left in mono, so the person sees the backend's limit before it refuses.
	type Props = {
		label: string;
		value?: string;
		helper?: string;
		error?: string;
		placeholder?: string;
		rows?: number;
		/** For machine text: a system prompt, JSON, a list of ids. */
		mono?: boolean;
		maxlength?: number;
		disabled?: boolean;
		required?: boolean;
		name?: string;
		oninput?: (value: string) => void;
	};
	let { label, value = $bindable(''), helper, error, placeholder, rows = 4, mono = false, maxlength, disabled = false, required = false, name, oninput }: Props = $props();

	const id = $props.id();
	const note = $derived(error ?? helper);
</script>

<div class={['flex w-full flex-col gap-1.5', disabled && 'opacity-50']}>
	<div class="flex items-baseline justify-between gap-3">
		<label for={id} class="text-ui font-medium text-text">{label}</label>
		{#if maxlength}<span aria-hidden="true" class="font-mono text-label text-text-3">{value.length}/{maxlength}</span>{/if}
	</div>
	<div
		class={[
			'flex w-full items-start gap-2 rounded-sm border bg-surface px-1.75 py-1.75 focus-within:shadow-focus',
			error ? 'border-danger' : disabled ? 'border-border' : 'border-border-strong focus-within:border-accent',
		]}
	>
		<textarea
			{id}
			{name}
			{rows}
			{placeholder}
			{disabled}
			{required}
			{maxlength}
			bind:value
			oninput={() => oninput?.(value)}
			aria-invalid={error ? 'true' : undefined}
			aria-describedby={note ? `${id}-note` : undefined}
			class={['min-w-0 flex-1 resize-y bg-transparent text-ui leading-normal text-text caret-accent outline-none placeholder:text-text-3 disabled:text-text-3', mono ? 'font-mono' : 'font-sans']}
		></textarea>
		{#if error}<span class="inline-flex pt-0.5 text-danger"><Icon icon={CircleAlert} size={14} /></span>{/if}
	</div>
	{#if note}<p id="{id}-note" class={['text-caption', error ? 'text-danger' : 'text-text-3']}>{note}</p>{/if}
</div>
