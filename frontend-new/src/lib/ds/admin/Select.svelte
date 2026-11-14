<script lang="ts" module>
	export type SelectOption = { value: string; label: string; disabled?: boolean };
</script>

<script lang="ts">
	import { ChevronDown, CircleAlert } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Admin → Select: one choice from a list too long for Segmented (a plan, a provider, a card
	// tone), in TextField's label · field · helper. It is the platform's own select, so the keyboard,
	// screen readers and a phone's picker work as people expect; only the box is drawn, with the
	// chevron in text-3.
	type Props = {
		label: string;
		options: SelectOption[];
		value?: string;
		helper?: string;
		error?: string;
		/** For machine values: model ids, environment names. */
		mono?: boolean;
		disabled?: boolean;
		required?: boolean;
		name?: string;
		onchange?: (value: string) => void;
	};
	let { label, options, value = $bindable(options[0]?.value ?? ''), helper, error, mono = false, disabled = false, required = false, name, onchange }: Props = $props();

	const id = $props.id();
	const note = $derived(error ?? helper);
</script>

<div class={['flex w-full flex-col gap-1.5', disabled && 'opacity-50']}>
	<label for={id} class="text-ui font-medium text-text">{label}</label>
	<div
		class={[
			'relative flex h-8.5 w-full items-center rounded-sm border bg-surface focus-within:shadow-focus',
			error ? 'border-danger' : disabled ? 'border-border' : 'border-border-strong focus-within:border-accent',
		]}
	>
		<select
			{id}
			{name}
			{disabled}
			{required}
			bind:value
			onchange={() => onchange?.(value)}
			aria-invalid={error ? 'true' : undefined}
			aria-describedby={note ? `${id}-note` : undefined}
			class={['h-full w-full min-w-0 cursor-pointer appearance-none truncate bg-transparent pr-7.5 pl-1.75 text-ui text-text outline-none disabled:cursor-default', mono ? 'font-mono' : 'font-sans']}
		>
			{#each options as option (option.value)}
				<option value={option.value} disabled={option.disabled}>{option.label}</option>
			{/each}
		</select>
		<span aria-hidden="true" class="pointer-events-none absolute right-2 inline-flex text-text-3">
			<Icon icon={error ? CircleAlert : ChevronDown} size={14} />
		</span>
	</div>
	{#if note}<p id="{id}-note" class={['text-caption', error ? 'text-danger' : 'text-text-3']}>{note}</p>{/if}
</div>
