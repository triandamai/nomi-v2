<!-- A choice between a few options (Chat responses → Decision). The crew's recommendation starts
     picked; confirming sends the choice as the person's answer, and the card stays as it was
     answered. What speaks for and against each option is written into its detail. -->
<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { useChatSend } from '$lib/chatSend';
	import DecisionCard from '$lib/components/ds/chat/DecisionCard.svelte';
	import type { DecisionView, ViewProps } from '$lib/views/types';

	let { view, answered = false }: ViewProps<DecisionView> = $props();

	const send = useChatSend();
	let picked = $state<string | null>(null);
	const done = $derived(answered || picked !== null || !send);

	const options = $derived(
		view.options.map((option) => {
			const points = [...(option.pros ?? []).map((pro) => `+ ${pro}`), ...(option.cons ?? []).map((con) => `− ${con}`)];
			const detail = [option.detail, points.join(' · ')].filter(Boolean).join(' — ');
			return { title: option.label, detail: detail || undefined, suggested: option.recommended };
		}),
	);

	function confirm(index: number) {
		const label = view.options[index]?.label;
		if (done || !label || !send) return;
		send(label);
		picked = label;
	}
</script>

<fieldset class="contents" disabled={done}>
	<DecisionCard question={view.question} {options} onconfirm={confirm} />
</fieldset>
{#if answered || picked}<p class="view-answered">{picked ? `${m.view_answered()} · ${picked}` : m.view_answered()}</p>{/if}

<style>
	.view-answered {
		margin: 6px 0 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
</style>
