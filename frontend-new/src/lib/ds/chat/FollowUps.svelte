<script lang="ts">
	import { CornerDownRight } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import SectionLabel from '../status/SectionLabel.svelte';

	// Chat responses → Follow-ups: optional next questions under an answer. Picking one asks it.
	let { items, onpick }: { items: string[]; onpick: (item: string) => void } = $props();
</script>

<section class="flex w-full flex-col gap-0.5">
	<SectionLabel label={m.followups_title()} />
	<ul class="flex flex-col gap-0.5">
		{#each items as item (item)}
			<li>
				<button
					type="button"
					class="flex w-full cursor-pointer items-center gap-2.5 rounded-xs py-1.75 text-left text-ui text-text-2 outline-offset-1 hover:text-text pointer-coarse:min-h-11"
					onclick={() => onpick(item)}
				>
					<span class="inline-flex shrink-0 text-text-3"><Icon icon={CornerDownRight} size={14} /></span>{item}
				</button>
			</li>
		{/each}
	</ul>
</section>
