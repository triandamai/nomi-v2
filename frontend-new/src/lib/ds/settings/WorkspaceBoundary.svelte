<script lang="ts">
	import { Info, Lock, Users } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// App / Settings / Workspace (wZm9B) → Side: where a workspace ends. SHARED HERE and ALWAYS
	// PERSONAL, each its mono label with an icon over its lines (14 apart), then the tip under a line.
	type Group = { title: string; items: string[] };
	type Props = { shared: Group; personal: Group; tip: string };
	let { shared, personal, tip }: Props = $props();
</script>

<aside class="flex flex-col gap-5">
	<div class="flex flex-col gap-3.5">
		{#each [{ group: shared, icon: Users }, { group: personal, icon: Lock }] as part (part.group.title)}
			<section class="flex flex-col gap-1.5">
				<h3 class="flex items-center gap-1.5 font-mono text-label tracking-label text-text-3 uppercase"><Icon icon={part.icon} size={12} />{part.group.title}</h3>
				<ul class="flex flex-col gap-1.5 text-caption text-text-2">
					{#each part.group.items as item (item)}<li>· {item}</li>{/each}
				</ul>
			</section>
		{/each}
	</div>
	<p class="flex gap-2 border-t border-border pt-4 text-caption text-text-3">
		<span class="inline-flex shrink-0 pt-px"><Icon icon={Info} size={14} /></span>{tip}
	</p>
</aside>
