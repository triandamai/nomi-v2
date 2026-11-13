<script lang="ts">
	import type { Component } from 'svelte';
	import { CircleAlert, type LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Auth / Sign in or create → Email Field: the sign-in screens' larger field. Label (13, 500), then
	// a 44 high field on surface with a 16px icon and 15px text (16 on a phone, so the page doesn't
	// zoom). Focus edges it in accent with the soft glow; an error edges it in danger and says what to
	// do in the helper's place. Inputs' TextField is 34 high and has no icon, so this is its own part.
	type Props = {
		label: string;
		value?: string;
		type?: 'text' | 'email' | 'password';
		name?: string;
		icon?: Component<LucideProps>;
		placeholder?: string;
		autocomplete?: HTMLInputElement['autocomplete'];
		helper?: string;
		error?: string;
		required?: boolean;
		/** Shown, not typed: a value that came with the link (an invite code). */
		readonly?: boolean;
		/** For machine values: codes, slugs. */
		mono?: boolean;
		/** Shows the focus state without focus (the gallery). */
		'data-focus'?: boolean;
	};
	let {
		label,
		value = $bindable(''),
		type = 'text',
		name,
		icon,
		placeholder,
		autocomplete,
		helper,
		error,
		required = false,
		readonly = false,
		mono = false,
		'data-focus': showFocus = false,
	}: Props = $props();

	const id = $props.id();
	const note = $derived(error ?? helper);
</script>

<div class="flex w-full flex-col gap-1.5">
	<label for={id} class="text-ui font-medium text-text">{label}</label>
	<div
		data-focus={showFocus || undefined}
		class={[
			'flex h-11 w-full items-center gap-2.5 rounded-sm border bg-surface px-2.75 focus-within:shadow-focus data-focus:border-accent data-focus:shadow-focus',
			error ? 'border-danger' : 'border-border-strong focus-within:border-accent',
		]}
	>
		{#if icon}<span class="inline-flex text-text-3"><Icon {icon} /></span>{/if}
		<input
			{id}
			{type}
			{name}
			{placeholder}
			{required}
			{readonly}
			{autocomplete}
			bind:value
			aria-invalid={error ? 'true' : undefined}
			aria-describedby={note ? `${id}-note` : undefined}
			class={[
				'min-w-0 flex-1 bg-transparent text-body text-text caret-accent outline-none placeholder:text-text-3 read-only:text-text-2 max-sm:text-phone',
				mono ? 'font-mono' : 'font-sans',
			]}
		/>
		{#if error}<span class="inline-flex text-danger"><Icon icon={CircleAlert} size={14} /></span>{/if}
	</div>
	{#if note}
		<p id="{id}-note" class={['text-caption', error ? 'text-danger' : 'text-text-3']}>{note}</p>
	{/if}
</div>
