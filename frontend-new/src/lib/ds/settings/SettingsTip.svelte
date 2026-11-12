<script lang="ts">
	import { CircleCheck, Info } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// App / Settings / General (m7lvY1) → Tip: one line about how the page works, under a stroke,
	// with the accent check, or the quiet info glyph for a word of explanation (Plan & usage).
	let { text, kind = 'check' }: { text: string; kind?: 'check' | 'info' } = $props();
</script>

<p class="flex items-start gap-2 border-t border-border pt-2.75 text-caption text-text-3">
	<span class={['inline-flex shrink-0', kind === 'check' ? 'text-accent' : 'text-text-3']}><Icon icon={kind === 'check' ? CircleCheck : Info} size={14} /></span>
	<span>{text}</span>
</p>
