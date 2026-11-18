<script lang="ts">
	import { ArrowUp } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Mascot from '../agents/Mascot.svelte';

	// Mobile / Ask Bar at the foot of a project's own screen (Flow / Mobile Projects — Detail panel,
	// kB2Dt; typing eiQ7z): on the page's ground under a top stroke, [10,16,4,16], the bar floating
	// in it: 52 tall and round, surface with a border-strong stroke and the float shadow, [0,6,0,14],
	// 10 apart: Koda's 20 mascot, the field (placeholder at 15 in text-3), and once something is
	// written the 40 round send (surface-2, arrow-up 18 in text) where the pen's mic sits. The mic is
	// left out until voice notes come to the composers (§9); what's typed is at 16pt, a point over
	// the pen's 15, so iOS doesn't zoom into the field. Enter sends.
	type Props = { placeholder: string; value?: string; onsend: (text: string) => void };
	let { placeholder, value = $bindable(''), onsend }: Props = $props();

	function send(event?: SubmitEvent) {
		event?.preventDefault();
		const text = value.trim();
		if (!text) return;
		value = '';
		onsend(text);
	}
</script>

<form class="border-t border-border bg-bg px-4 pt-2.25 pb-1" onsubmit={send}>
	<div class="flex h-13 items-center gap-2.5 rounded-full bg-surface pr-1.5 pl-3.5 shadow-float outline -outline-offset-1 outline-border-strong">
		<Mascot agent="koda" size={20} />
		<input
			bind:value
			{placeholder}
			aria-label={placeholder}
			enterkeyhint="send"
			autocomplete="off"
			class="h-full min-w-0 flex-1 bg-transparent text-phone text-text caret-accent outline-none placeholder:text-body placeholder:text-text-3"
		/>
		{#if value.trim()}
			<button type="submit" aria-label={m.composer_send()} class="inline-flex size-10 shrink-0 cursor-pointer items-center justify-center rounded-full bg-surface-2 text-text outline-offset-1">
				<Icon icon={ArrowUp} size={18} />
			</button>
		{/if}
	</div>
</form>
