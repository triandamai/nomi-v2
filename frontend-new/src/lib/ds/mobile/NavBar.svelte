<script lang="ts">
	import { ChevronLeft, Ellipsis } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// Mobile / Nav Bar: back and the chat's menu on 44pt targets, the chat's title and who answers
	// in it, centred, 52pt tall.
	type Props = { title: string; who?: { agent: CrewId; name: string }; onback: () => void; onmenu?: () => void };
	let { title, who, onback, onmenu }: Props = $props();
</script>

<header class="flex h-13 w-full items-center gap-1 border-b border-border px-1.5">
	<button type="button" aria-label={m.mobile_back()} class="inline-flex size-11 shrink-0 cursor-pointer items-center justify-center rounded-full text-text outline-offset-1" onclick={onback}>
		<Icon icon={ChevronLeft} size={26} />
	</button>
	<div class="flex min-w-0 flex-1 flex-col items-center gap-0.5">
		<h1 class="max-w-full truncate text-phone leading-tight font-semibold text-text">{title}</h1>
		{#if who}
			<span class="inline-flex items-center gap-1.25 text-caption text-text-3"><Mascot agent={who.agent} size={16} />{who.name}</span>
		{/if}
	</div>
	{#if onmenu}
		<button type="button" aria-label={m.chat_options()} class="inline-flex size-11 shrink-0 cursor-pointer items-center justify-center rounded-full text-text outline-offset-1" onclick={onmenu}>
			<Icon icon={Ellipsis} size={22} />
		</button>
	{:else}
		<span class="size-11 shrink-0"></span>
	{/if}
</header>
