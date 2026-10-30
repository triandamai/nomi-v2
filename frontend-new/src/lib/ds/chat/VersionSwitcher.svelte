<script lang="ts">
	import { ChevronLeft, ChevronRight } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Chat responses → Version Switcher: edits branch instead of overwriting, so a message can have
	// versions; this steps between them ("‹ 2 / 2 ›").
	type Props = { current: number; total: number; onprev: () => void; onnext: () => void };
	let { current, total, onprev, onnext }: Props = $props();
</script>

<span class="inline-flex items-center gap-0.5">
	<button
		type="button"
		aria-label={m.version_previous()}
		disabled={current <= 1}
		class="inline-flex cursor-pointer rounded-xs p-0.5 text-text-2 outline-offset-1 hover:enabled:text-text disabled:cursor-default disabled:text-text-3 pointer-coarse:size-11 pointer-coarse:items-center pointer-coarse:justify-center"
		onclick={onprev}><Icon icon={ChevronLeft} size={12} /></button
	>
	<span class="font-mono text-label text-text-2" aria-label={m.version_label({ current: String(current), total: String(total) })}>{current} / {total}</span>
	<button
		type="button"
		aria-label={m.version_next()}
		disabled={current >= total}
		class="inline-flex cursor-pointer rounded-xs p-0.5 text-text-2 outline-offset-1 hover:enabled:text-text disabled:cursor-default disabled:text-text-3 pointer-coarse:size-11 pointer-coarse:items-center pointer-coarse:justify-center"
		onclick={onnext}><Icon icon={ChevronRight} size={12} /></button
	>
</span>
