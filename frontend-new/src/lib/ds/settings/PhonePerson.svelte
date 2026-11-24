<script lang="ts">
	import { ChevronRight } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Flow / Mobile — Home, Inbox & You, step 3 (O1SXw) → Person (HUrPH): who's signed in, opening their
	// profile. A card (surface, border, radius 16, inset 14, 12 apart): the 52 avatar (their photo, else
	// initials at 17/600), the name at 17/600 and the line under it at 14, and a chevron.
	type Props = { name: string; detail: string; avatar?: string; href: string };
	let { name, detail, avatar, href }: Props = $props();
	const initials = $derived(name.split(/\s+/).filter(Boolean).map((word) => word.charAt(0)).join('').slice(0, 2).toUpperCase());
</script>

<a {href} class="flex items-center gap-3 rounded-2xl bg-surface p-3.5 outline -outline-offset-1 outline-border focus-visible:outline-2 focus-visible:outline-focus">
	{#if avatar}
		<img src={avatar} alt="" class="size-13 shrink-0 rounded-full object-cover" />
	{:else}
		<span aria-hidden="true" class="inline-flex size-13 shrink-0 items-center justify-center rounded-full bg-surface-2 text-phone-title font-semibold text-text">{initials}</span>
	{/if}
	<span class="flex min-w-0 flex-1 flex-col gap-0.5">
		<span class="truncate text-phone-title font-semibold text-text">{name}</span>
		<span class="truncate text-phone-sm text-text-3">{detail}</span>
	</span>
	<span class="inline-flex shrink-0 text-text-3"><Icon icon={ChevronRight} size={16} /></span>
</a>
