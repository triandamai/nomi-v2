<script lang="ts">
	import type { Component } from 'svelte';
	import { ArrowUpRight, Bookmark, Minus, Package, Plus, ShoppingBasket, type LucideProps } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';

	// Chat responses → Cart: you check out at the store; Nomi only records it. Real prices with the
	// time they were read, what's out of stock said plainly, a swap offered when there is one.
	type Item = {
		name: string;
		/** Size or pack, e.g. "1 tray · 10 butir". */
		detail?: string;
		icon?: Component<LucideProps>;
		quantity: number;
		/** Formatted by the caller ("58.000"); "—" when it can't be bought. */
		price: string;
		/** False when the store is out of it. */
		available?: boolean;
		/** What to get instead, e.g. "Swap for tongkol". */
		swap?: string;
	};
	type Props = {
		title: string;
		store: string;
		items: Item[];
		subtotal: string;
		delivery?: string;
		total: string;
		note?: string;
		checkoutHref: string;
		onquantity?: (index: number, quantity: number) => void;
		onswap?: (index: number) => void;
		onsave?: () => void;
	};
	let { title, store, items, subtotal, delivery, total, note, checkoutHref, onquantity, onswap, onsave }: Props = $props();
</script>

<section class="flex w-full flex-col overflow-hidden rounded-md border border-border-strong bg-bg">
	<header class="flex items-center gap-2.5 border-b border-border bg-surface px-3.5 py-2.5">
		<span class="inline-flex text-text-2"><Icon icon={ShoppingBasket} size={14} /></span>
		<p class="min-w-0 flex-1 truncate text-ui font-medium text-text">{title}</p>
		<span class="font-mono text-label text-text-3">{store} · {items.length === 1 ? m.cart_item_one() : m.cart_items({ count: String(items.length) })}</span>
	</header>
	<ul>
		{#each items as item, index (index)}
			{@const available = item.available !== false}
			<li class={['flex flex-wrap items-center gap-3 px-3.5 py-2.5', index > 0 && 'border-t border-border']}>
				<span class={['inline-flex size-9 shrink-0 items-center justify-center rounded-sm bg-surface-2', available ? 'text-text-2' : 'text-text-3']}>
					<Icon icon={item.icon ?? Package} size={16} />
				</span>
				<span class="flex min-w-0 flex-1 basis-32 flex-col gap-0.5">
					<span class={['truncate text-ui', available ? 'text-text' : 'text-text-3']}>{item.name}</span>
					{#if !available}
						<span class="font-mono text-label text-warning">{m.cart_out_of_stock()}</span>
						{#if item.swap && onswap}
							<button type="button" class="cursor-pointer self-start rounded-xs text-left text-caption text-text-2 underline decoration-border-strong underline-offset-2 outline-offset-1 hover:text-text" onclick={() => onswap(index)}>{item.swap}</button>
						{:else if item.swap}
							<span class="text-caption text-text-2">{item.swap}</span>
						{/if}
					{:else if item.detail}
						<span class="truncate font-mono text-label text-text-3">{item.detail}</span>
					{/if}
				</span>
				<span class="ml-auto flex items-center gap-3">
					<span role="group" aria-label={m.cart_quantity({ item: item.name })} class={['inline-flex h-6.5 shrink-0 items-center rounded-sm border border-border-strong', !available && 'opacity-40']}>
						<button
							type="button"
							aria-label={m.cart_less({ item: item.name })}
							disabled={!available || !onquantity || item.quantity <= 0}
							class="inline-flex size-6 cursor-pointer items-center justify-center rounded-xs text-text-2 outline-offset-1 hover:enabled:text-text disabled:cursor-default pointer-coarse:size-11"
							onclick={() => onquantity?.(index, item.quantity - 1)}><Icon icon={Minus} size={12} /></button
						>
						<span class="w-5 text-center font-mono text-label text-text">{item.quantity}</span>
						<button
							type="button"
							aria-label={m.cart_more({ item: item.name })}
							disabled={!available || !onquantity}
							class="inline-flex size-6 cursor-pointer items-center justify-center rounded-xs text-text-2 outline-offset-1 hover:enabled:text-text disabled:cursor-default pointer-coarse:size-11"
							onclick={() => onquantity?.(index, item.quantity + 1)}><Icon icon={Plus} size={12} /></button
						>
					</span>
					<span class={['w-18 shrink-0 text-right font-mono text-caption', available ? 'text-text' : 'text-text-3']}>{item.price}</span>
				</span>
			</li>
		{/each}
	</ul>
	<dl class="flex flex-col gap-1.5 border-t border-border bg-surface px-3.5 py-2.5">
		<div class="flex items-center justify-between">
			<dt class="text-ui text-text-2">{m.cart_subtotal()}</dt>
			<dd class="font-mono text-caption text-text">{subtotal}</dd>
		</div>
		{#if delivery}
			<div class="flex items-center justify-between">
				<dt class="text-ui text-text-2">{m.cart_delivery()}</dt>
				<dd class="font-mono text-caption text-text">{delivery}</dd>
			</div>
		{/if}
		<div class="flex items-center justify-between font-medium">
			<dt class="text-ui text-text">{m.cart_total()}</dt>
			<dd class="font-mono text-caption text-text">{total}</dd>
		</div>
	</dl>
	<footer class="flex flex-wrap items-center gap-2 border-t border-border px-3.5 py-2.5">
		<p class="min-w-0 flex-1 basis-48 text-caption text-text-3">{note}</p>
		<span class="ml-auto flex flex-wrap items-center justify-end gap-2">
			{#if onsave}<Button label={m.cart_save()} variant="ghost" iconStart={Bookmark} onclick={onsave} />{/if}
			<Button label={m.cart_checkout({ store })} href={checkoutHref} external iconEnd={ArrowUpRight} />
		</span>
	</footer>
</section>
