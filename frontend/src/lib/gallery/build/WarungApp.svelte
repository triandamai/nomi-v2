<script lang="ts">
	import { Minus, Plus } from '@lucide/svelte';
	import Icon from '$lib/components/ds/foundations/Icon.svelte';

	// Mobile / Warung App: the app Koda built in the story (warung-kopi-menu), as the pen draws it
	// at phone size. Sample content for the phone preview and the live app, not Nomi's UI: two
	// blocks for a column parent, the menu and the cart bar pinned under it.
	const menu = [
		{ name: 'Kopi susu gula aren', price: 'Rp 18.000', quantity: 2 },
		{ name: 'Es kopi Bu Siti', price: 'Rp 20.000', badge: 'best seller' },
		{ name: 'Kopi tubruk', price: 'Rp 10.000' },
		{ name: 'Teh tarik', price: 'Rp 12.000' },
	];
</script>

<div class="flex min-h-0 flex-1 flex-col gap-3.5 px-4.5 pt-2">
	<div class="flex flex-col gap-1">
		<p class="sample-title font-semibold text-text">Warung Kopi Bu Siti</p>
		<p class="text-caption text-text-2">Jl. Cihampelas 41 · pickup 07:00–11:00</p>
	</div>
	<div class="flex gap-1.5">
		<span class="inline-flex h-7.5 items-center rounded-full bg-text px-3.5 text-ui font-medium text-bg">Kopi</span>
		<span class="inline-flex h-7.5 items-center rounded-full border border-border-strong px-3.5 text-ui text-text-2">Makanan</span>
	</div>
	<ul>
		{#each menu as item (item.name)}
			<li class="flex h-16 items-center gap-2.5 border-b border-border">
				<span class="flex min-w-0 flex-1 flex-col gap-0.75">
					<span class="text-ui font-medium text-text">{item.name}</span>
					<span class="flex items-center gap-1.5">
						<span class="text-caption text-text-2">{item.price}</span>
						{#if item.badge}<span class="rounded-xs bg-accent-bg px-1.5 py-px font-mono text-micro text-accent">{item.badge}</span>{/if}
					</span>
				</span>
				{#if item.quantity}
					<span class="inline-flex h-8 items-center gap-2.5 rounded-full bg-surface-2 px-2 text-ui font-medium"><Icon icon={Minus} size={14} />{item.quantity}<Icon icon={Plus} size={14} /></span>
				{:else}
					<span class="inline-flex size-8 items-center justify-center rounded-full border border-border-strong"><Icon icon={Plus} size={14} /></span>
				{/if}
			</li>
		{/each}
	</ul>
</div>
<div class="border-t border-border bg-bg px-4.5 pt-3 pb-5.5">
	<p class="flex h-11.5 items-center justify-between gap-1 rounded-full bg-text px-4 text-bg">
		<span class="text-caption whitespace-nowrap">2 items · pickup 07:30</span><span class="text-ui font-semibold whitespace-nowrap">Pre-order · Rp 36.000</span>
	</p>
</div>

<style>
	/* The example app's own heading size (it's the person's app, not Nomi's UI). */
	.sample-title {
		font-size: 22px;
		line-height: 28px;
	}
</style>
