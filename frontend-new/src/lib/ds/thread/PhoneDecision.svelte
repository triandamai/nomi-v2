<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import PhoneOption from '../mobile/PhoneOption.svelte';
	import PhoneButton from '../mobile/PhoneButton.svelte';

	// Flow / Mobile Chat — Plan → approve → act, step 2 (gQ44i): a choice the crew asks for in the
	// thread, on a phone: the options as Mobile / Option rows 12 apart, the suggested one first,
	// picked and tagged RECOMMENDED, then "Use {option}" across the width, which answers with it.
	type Option = { title: string; detail?: string; suggested?: boolean };
	type Props = { question: string; options: Option[]; onconfirm: (index: number) => void };
	let { question, options, onconfirm }: Props = $props();

	const id = $props.id();
	// The suggested one first, as the pen draws it; the index given back is the option's own.
	const order = $derived(options.map((_, index) => index).sort((a, b) => Number(Boolean(options[b].suggested)) - Number(Boolean(options[a].suggested))));
	const start = () => options.findIndex((option) => option.suggested);
	let chosen = $state(start());
</script>

<div class="flex w-full flex-col gap-3">
	<div role="radiogroup" aria-label={question} class="flex flex-col gap-3">
		{#each order as index, at (index)}
			{@const option = options[index]}
			<PhoneOption name="{id}-decision" key={at + 1} title={option.title} detail={option.detail} recommended={option.suggested} checked={chosen === index} onselect={() => (chosen = index)} />
		{/each}
	</div>
	<PhoneButton label={chosen >= 0 ? m.phone_decision_use({ option: options[chosen].title }) : m.phone_decision_use({ option: '' }).trim()} disabled={chosen < 0} onclick={() => chosen >= 0 && onconfirm(chosen)} />
</div>
