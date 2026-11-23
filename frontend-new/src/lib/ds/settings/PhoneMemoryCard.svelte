<script lang="ts">
	import { Ellipsis } from '@lucide/svelte';
	import StrengthBars from './StrengthBars.svelte';

	// Flow / Mobile — You & Settings, step 17 (a kind of memory) → Memory: one memory as a card
	// (surface, border, radius 14, [12,6,12,14]): what the crew remembers at 14, then its strength
	// and who learned it when in mono; a 44 ⋯ at the right opens what can be done with it. One the
	// crew isn't sure of asks so under it.
	type Props = {
		content: string;
		strength: { level: 1 | 2 | 3 | 4 | 5; name: string };
		meta: string;
		/** "Is this still right?" when the crew isn't sure of it. */
		check?: string;
		more: string;
		busy?: boolean;
		onmore: () => void;
	};
	let { content, strength, meta, check, more, busy = false, onmore }: Props = $props();
</script>

<article aria-label={content} class={['flex items-start gap-1 rounded-xl bg-surface py-3 pr-1.5 pl-3.5 outline -outline-offset-1 outline-border', busy && 'opacity-60']}>
	<div class="flex min-w-0 flex-1 flex-col gap-2 pt-0.5">
		<p class="text-phone-sm leading-firm text-text">{content}</p>
		<div class="flex flex-wrap items-end gap-x-3 gap-y-1">
			<StrengthBars level={strength.level} name={strength.name} />
			<span class="min-w-0 font-mono text-label text-text-3">{meta}</span>
		</div>
		{#if check}<p class="text-phone-sm text-warning">{check}</p>{/if}
	</div>
	<button
		type="button"
		aria-label={more}
		disabled={busy}
		onclick={onmore}
		class="-mt-2 inline-flex size-11 shrink-0 cursor-pointer items-center justify-center rounded-full text-text-2 outline-offset-1 focus-visible:outline-2 focus-visible:outline-focus disabled:cursor-default"
	>
		<Ellipsis size={18} aria-hidden="true" />
	</button>
</article>
