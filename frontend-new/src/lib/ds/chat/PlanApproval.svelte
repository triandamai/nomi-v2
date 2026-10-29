<script lang="ts">
	import { Play, Send } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';
	import ChoiceOption from './ChoiceOption.svelte';

	// Chat responses → Plan Approval: how much of the plan may run on its own, the recommendation
	// picked first, or keep planning with what you want changed. The button says what will happen.
	type Option = { title: string; detail?: string; recommended?: boolean };
	type Props = { options: Option[]; onrun: (choice: { choice: number } | { keepPlanning: string }) => void };
	let { options, onrun }: Props = $props();

	const id = $props.id();
	let chosen = $state(-1);
	let changes = $state('');

	const value = $derived(chosen >= 0 ? chosen : Math.max(0, options.findIndex((option) => option.recommended)));
	const keep = $derived(value === options.length);

	function run() {
		onrun(keep ? { keepPlanning: changes.trim() } : { choice: value });
	}

	function onkeydown(event: KeyboardEvent) {
		const target = event.target as HTMLElement;
		const typing = target instanceof HTMLInputElement && target.type === 'text';
		if (event.key === 'Enter' && !(target instanceof HTMLButtonElement)) {
			event.preventDefault();
			run();
		} else if (!typing && /^[1-9]$/.test(event.key) && Number(event.key) <= options.length + 1) {
			event.preventDefault();
			chosen = Number(event.key) - 1;
		}
	}
</script>

<!-- Keys work anywhere inside the card; each control is also reachable on its own. -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<section class="flex w-full flex-col overflow-hidden rounded-md border border-border-strong bg-bg" aria-labelledby="{id}-title" {onkeydown}>
	<header class="flex items-center gap-2.5 border-b border-border bg-surface px-3.5 py-2.5">
		<span class="inline-flex text-text-2"><Icon icon={Play} size={14} /></span>
		<p id="{id}-title" class="min-w-0 flex-1 text-ui font-medium text-text">{m.plan_approval_title()}</p>
	</header>
	<div role="radiogroup" aria-labelledby="{id}-title" class="flex flex-col gap-1.5 p-3.5">
		{#each options as option, index (index)}
			<ChoiceOption
				type="radio"
				name="{id}-mode"
				key={index + 1}
				checked={index === value}
				title={option.title}
				detail={option.detail}
				tag={option.recommended ? m.choice_recommended() : undefined}
				onchange={(checked) => checked && (chosen = index)}
			/>
		{/each}
		<ChoiceOption
			type="radio"
			name="{id}-mode"
			key={options.length + 1}
			checked={keep}
			title={m.plan_approval_keep()}
			onchange={(checked) => checked && (chosen = options.length)}
		>
			<input
				type="text"
				aria-label={m.plan_approval_keep_placeholder()}
				placeholder={m.plan_approval_keep_placeholder()}
				bind:value={changes}
				oninput={() => (chosen = options.length)}
				class="h-8 w-full rounded-sm border border-border bg-surface px-2.5 text-caption text-text placeholder:text-text-3"
			/>
		</ChoiceOption>
	</div>
	<footer class="flex flex-wrap items-center gap-2 border-t border-border px-3.5 py-2.5">
		<span class="min-w-0 flex-1 basis-40 font-mono text-label text-text-3">{m.plan_approval_keys({ count: String(options.length + 1) })}</span>
		<span class="ml-auto"><Button label={keep ? m.plan_approval_send() : m.plan_approval_run()} iconEnd={keep ? Send : Play} disabled={keep && !changes.trim()} onclick={run} /></span>
	</footer>
</section>
