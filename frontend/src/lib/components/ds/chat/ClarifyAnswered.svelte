<script lang="ts">
	import { Check, ChevronRight } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Chat responses → Clarify / Answered: the questions folded to one line once answered, so the
	// thread stays readable. Opens them again.
	type Props = { count: number; summary: string; onopen: () => void };
	let { count, summary, onopen }: Props = $props();
</script>

<button
	type="button"
	class="flex w-full cursor-pointer items-center gap-2 rounded-sm border border-border px-2.75 py-1.75 text-left outline-offset-2 transition-colors hover:bg-surface pointer-coarse:min-h-11"
	onclick={onopen}
>
	<span class="inline-flex shrink-0 text-accent"><Icon icon={Check} size={14} /></span>
	<span class="shrink-0 text-caption text-text-2">{count === 1 ? m.clarify_answered_one() : m.clarify_answered({ count: String(count) })}</span>
	<span class="min-w-0 flex-1 truncate font-mono text-label text-text-3">{summary}</span>
	<span class="inline-flex shrink-0 text-text-3"><Icon icon={ChevronRight} size={12} /></span>
</button>
