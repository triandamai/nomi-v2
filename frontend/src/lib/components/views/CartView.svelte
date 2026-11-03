<!-- A basket at a store (Chat responses → Cart): the person checks out at the store itself; Nomi
     buys nothing. Prices are the ones the crew read from the store, totals are worked out here as
     quantities change. Saving it, or taking a swap, is asked of the crew in the chat. -->
<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { useChatSend } from '$lib/chatSend';
	import { formatValue } from '$lib/views/format';
	import Cart from '$lib/components/ds/chat/Cart.svelte';
	import type { CartView, ViewProps } from '$lib/views/types';

	let { view, answered = false }: ViewProps<CartView> = $props();

	const send = useChatSend();
	const initial = () => view.items.map((item) => (item.available === false ? 0 : (item.quantity ?? 1)));
	let quantities = $state(initial());

	const money = (amount: number) => formatValue(amount, 'money', getLocale());
	const items = $derived(
		view.items.map((item, index) => ({
			name: item.name,
			detail: item.detail,
			quantity: quantities[index],
			price: item.available === false || item.price === undefined ? '—' : money(item.price * quantities[index]),
			available: item.available !== false,
			swap: item.available === false ? item.swap : undefined,
		})),
	);
	const subtotal = $derived(view.items.reduce((sum, item, index) => sum + (item.available === false ? 0 : (item.price ?? 0) * quantities[index]), 0));
	const total = $derived(subtotal + (view.delivery ?? 0));

	function ask(text: string) {
		if (!answered && send) send(text);
	}
</script>

<Cart
	title={view.title}
	store={view.store}
	{items}
	subtotal={money(subtotal)}
	delivery={view.delivery === undefined ? undefined : money(view.delivery)}
	total={money(total)}
	note={view.note}
	checkoutHref={view.checkout_url}
	onquantity={(index, quantity) => (quantities[index] = Math.max(0, quantity))}
	onswap={send && !answered ? (index) => ask(m.view_cart_swap({ item: view.items[index].name, swap: view.items[index].swap ?? '' })) : undefined}
	onsave={send && !answered
		? () =>
				ask(
					m.view_cart_save({
						title: view.title,
						items: view.items.flatMap((item, index) => (quantities[index] > 0 ? [`${quantities[index]} × ${item.name}`] : [])).join(', '),
					}),
				)
		: undefined}
/>
