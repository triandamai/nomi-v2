<script lang="ts">
	import { CircleAlert } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Design System / Nomi → Inputs: label · field · helper. The field sits on surface with
	// border-strong; focus adds the 3px focus ring; an error swaps the border and takes the
	// helper's place to say what to do next.
	type Props = {
		label: string;
		value?: string;
		helper?: string;
		error?: string;
		placeholder?: string;
		/** For machine values: slugs, keys, URLs. */
		mono?: boolean;
		disabled?: boolean;
		required?: boolean;
		type?: 'text' | 'email' | 'url' | 'password';
		name?: string;
		autocomplete?: HTMLInputElement['autocomplete'];
		oninput?: (value: string) => void;
		/** Shows the focus state without focus (the gallery). */
		'data-focus'?: boolean;
	};
	let {
		label,
		value = $bindable(''),
		helper,
		error,
		placeholder,
		mono = false,
		disabled = false,
		required = false,
		type = 'text',
		name,
		autocomplete,
		oninput,
		'data-focus': showFocus = false,
	}: Props = $props();

	const id = $props.id();
	const note = $derived(error ?? helper);
</script>

<div class={['flex w-full flex-col gap-1.5', disabled && 'opacity-50']}>
	<label for={id} class="text-ui font-medium text-text">{label}</label>
	<div
		data-focus={showFocus || undefined}
		class={[
			'flex h-8.5 w-full items-center gap-2 rounded-sm border bg-surface px-2 focus-within:shadow-focus data-focus:border-accent data-focus:shadow-focus',
			error ? 'border-danger' : disabled ? 'border-border' : 'border-border-strong focus-within:border-accent',
		]}
	>
		<input
			{id}
			{type}
			{name}
			{placeholder}
			{disabled}
			{required}
			{autocomplete}
			bind:value
			oninput={() => oninput?.(value)}
			aria-invalid={error ? 'true' : undefined}
			aria-describedby={note ? `${id}-note` : undefined}
			class={['min-w-0 flex-1 bg-transparent text-ui caret-accent outline-none placeholder:text-text-3 disabled:text-text-3', mono ? 'font-mono' : 'font-sans', 'text-text']}
		/>
		{#if error}<span class="inline-flex text-danger"><Icon icon={CircleAlert} size={14} /></span>{/if}
	</div>
	{#if note}
		<p id="{id}-note" class={['text-caption', error ? 'text-danger' : 'text-text-3']}>{note}</p>
	{/if}
</div>
