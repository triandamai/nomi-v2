<script lang="ts">
	import { Check, X } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import IconButton from '../actions/IconButton.svelte';

	// Chat responses → Voice Recorder, in place of the composer while recording: the real input
	// level as bars (the latest ones fading in), the time so far and the language it transcribes
	// in. Discard, or use it (it becomes text you can edit before sending).
	type Props = {
		elapsed: string;
		/** Input levels, 0 to 1, oldest first. */
		levels: number[];
		/** How many of the last levels are still settling, shown dimmer. */
		fading?: number;
		language: string;
		oncancel: () => void;
		ondone: () => void;
	};
	let { elapsed, levels, fading = 0, language, oncancel, ondone }: Props = $props();

	const height = (level: number) => Math.round(4 + Math.min(1, Math.max(0, level)) * 24);
</script>

<div role="group" aria-label={m.voice_recording()} class="flex w-full items-center gap-3 rounded-md bg-surface outline -outline-offset-1 outline-border-strong px-3 py-2.5">
	<IconButton icon={X} label={m.voice_cancel()} variant="ghost" onclick={oncancel} />
	<span aria-hidden="true" class="size-2 shrink-0 animate-pulse rounded-full bg-danger motion-reduce:animate-none"></span>
	<span class="font-mono text-label text-text">{elapsed}</span>
	<span aria-hidden="true" class="flex h-7 min-w-0 flex-1 items-center gap-0.75 overflow-hidden">
		{#each levels as level, index (index)}
			<span data-bar class={['w-0.75 shrink-0 rounded-full', index >= levels.length - fading ? 'bg-border-strong' : 'bg-text-2']} style="height: {height(level)}px"></span>
		{/each}
	</span>
	<span class="hidden font-mono text-label text-text-3 sm:inline">{language}</span>
	<IconButton icon={Check} label={m.voice_done()} variant="send" onclick={ondone} />
</div>
