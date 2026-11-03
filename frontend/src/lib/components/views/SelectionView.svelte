<!-- Several of many (Chat responses → Selection): the person ticks what they want, with a running
     count and total of what's picked; confirming sends the picked names as their answer. -->
<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { useChatSend } from '$lib/chatSend';
	import { crewId } from '$lib/crewId';
	import { formatValue } from '$lib/views/format';
	import SelectionCard from '$lib/components/ds/chat/SelectionCard.svelte';
	import type { SelectionView, ViewProps } from '$lib/views/types';

	let { view, agent = null, answered = false }: ViewProps<SelectionView> = $props();

	const send = useChatSend();
	const initial = () => view.items.map((item) => item.selected ?? false);
	let picked = $state(initial());
	let sent = $state<string | null>(null);
	const done = $derived(answered || sent !== null || !send);

	const money = (amount: number) => formatValue(amount, 'money', getLocale());
	const items = $derived(view.items.map((item) => ({ name: item.name, detail: item.detail, amount: item.amount === undefined ? undefined : money(item.amount) })));
	const priced = $derived(view.items.some((item) => item.amount !== undefined));
	const total = $derived.by(() => {
		if (!priced) return undefined;
		const sum = view.items.reduce((acc, item, index) => acc + (picked[index] ? (item.amount ?? 0) : 0), 0);
		return view.per ? m.view_selection_per({ amount: money(sum), per: view.per }) : money(sum);
	});

	function confirm(indices: number[]) {
		if (done || !send || indices.length === 0) return;
		const text = m.view_selection_reply({ items: indices.map((index) => view.items[index].name).join(', ') });
		send(text);
		sent = text;
	}
</script>

<fieldset class="contents" disabled={done}>
	<SelectionCard
		agent={crewId(agent)}
		question={view.question}
		{items}
		selected={picked}
		{total}
		confirmLabel={view.confirm_label ?? m.view_selection_confirm()}
		onchange={(next) => (picked = next)}
		onconfirm={confirm}
	/>
</fieldset>
{#if done && (answered || sent)}<p class="view-answered">{sent ?? m.view_answered()}</p>{/if}

<style>
	.view-answered {
		margin: 6px 0 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
</style>
