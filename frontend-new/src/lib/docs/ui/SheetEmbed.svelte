<script lang="ts">
	import type { Snippet } from 'svelte';
	import { ArrowUpRight } from '@lucide/svelte';
	import Icon from '$lib/ds/foundations/Icon.svelte';
	import { docHref } from '../href';

	// A section of a design sheet, shown in the docs at the sheet's own size (1472 wide, as the pen
	// draws it, inside its 64 margins), so it scrolls sideways on narrower screens rather than reflowing.
	type Props = { sheet: 'nomi' | 'chat' | 'build'; id: string; label: string; children: Snippet };
	let { sheet, id, label, children }: Props = $props();
	const names = { nomi: 'Nomi', chat: 'Chat responses', build: 'Build' };
</script>

<figure class="flex flex-col overflow-hidden rounded-md outline -outline-offset-1 outline-border">
	<figcaption class="flex flex-wrap items-center gap-x-3 gap-y-1 border-b border-border bg-surface px-4 py-2.5">
		<span class="font-mono text-label text-text-3">{names[sheet]} sheet · {label} · 1:1 with the pen</span>
		<a href={docHref(`/design-system/${sheet}`, id)} class="ml-auto inline-flex items-center gap-1 text-caption text-text-2 hover:text-text">
			Open in the sheet <Icon icon={ArrowUpRight} size={12} />
		</a>
	</figcaption>
	<div class="overflow-x-auto bg-bg">
		<div class="w-400 p-16">{@render children()}</div>
	</div>
</figure>
