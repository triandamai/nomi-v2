<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import ChoiceOption from './ChoiceOption.svelte';

	// How a plan may run, as numbered choices with the recommendation picked first, and a last one
	// to keep planning with what you want changed (typing in it picks it). Shared by the Plan
	// Approval and the Build Plan. `value` is the chosen index; options.length means keep planning.
	type Option = { title: string; detail?: string; recommended?: boolean };
	type Props = {
		name: string;
		label: string;
		options: Option[];
		/** Who the changes go to, e.g. "Koda". */
		who: string;
		value?: number;
		changes?: string;
	};
	let { name, label, options, who, value = $bindable(-1), changes = $bindable('') }: Props = $props();

	const chosen = $derived(value >= 0 ? value : Math.max(0, options.findIndex((option) => option.recommended)));
	const keep = $derived(chosen === options.length);
</script>

<div role="radiogroup" aria-label={label} class="flex flex-col gap-1.5">
	{#each options as option, index (index)}
		<ChoiceOption
			type="radio"
			{name}
			key={index + 1}
			checked={index === chosen}
			title={option.title}
			detail={option.detail}
			tag={option.recommended ? m.choice_recommended() : undefined}
			onchange={(checked) => checked && (value = index)}
		/>
	{/each}
	<ChoiceOption type="radio" {name} key={options.length + 1} checked={keep} title={m.plan_approval_keep()} onchange={(checked) => checked && (value = options.length)}>
		<input
			type="text"
			aria-label={m.plan_approval_keep_placeholder({ name: who })}
			placeholder={m.plan_approval_keep_placeholder({ name: who })}
			bind:value={changes}
			oninput={() => (value = options.length)}
			class="h-8 w-full rounded-sm border border-border bg-surface px-2.25 text-caption text-text placeholder:text-text-3"
		/>
	</ChoiceOption>
</div>
