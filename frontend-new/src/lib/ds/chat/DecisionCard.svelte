<script lang="ts">
	import { Check, Split } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';
	import ChoiceOption from './ChoiceOption.svelte';

	// Chat responses → Decision: one of many, each with its tradeoff written out. The crew's
	// suggestion starts picked; the button names what you're choosing. You can always answer in
	// your own words in the composer instead.
	type Option = { title: string; detail?: string; suggested?: boolean };
	type Props = { question: string; options: Option[]; onconfirm: (index: number) => void };
	let { question, options, onconfirm }: Props = $props();

	const id = $props.id();
	let chosen = $state(-1);
	const value = $derived(chosen >= 0 ? chosen : Math.max(0, options.findIndex((option) => option.suggested)));
</script>

<section class="flex w-full flex-col overflow-hidden rounded-md bg-bg outline -outline-offset-1 outline-border-strong">
	<header class="flex items-center gap-2.5 border-b border-border bg-surface px-3.5 pt-2.5 pb-2.25">
		<span class="inline-flex text-text-2"><Icon icon={Split} size={14} /></span>
		<p id="{id}-question" class="min-w-0 flex-1 text-ui font-medium text-text">{question}</p>
	</header>
	<div role="radiogroup" aria-labelledby="{id}-question" class="flex flex-col gap-2 p-3.5">
		{#each options as option, index (index)}
			<ChoiceOption
				type="radio"
				name="{id}-choice"
				control="start"
				checked={index === value}
				title={option.title}
				detail={option.detail}
				tag={option.suggested ? m.decision_suggested() : undefined}
				onchange={(checked) => checked && (chosen = index)}
			/>
		{/each}
	</div>
	<footer class="flex flex-wrap items-center gap-2 border-t border-border px-3.5 pt-2.25 pb-2.5">
		<span class="min-w-0 flex-1 basis-40 text-caption text-text-3">{m.decision_own()}</span>
		<span class="ml-auto"><Button label={m.decision_use({ choice: options[value]?.title ?? '' })} iconEnd={Check} onclick={() => onconfirm(value)} /></span>
	</footer>
</section>
