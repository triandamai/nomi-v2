<script lang="ts">
	import { FileCode, X } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Build → Code quote: lines picked in the code view, quoted in the composer: the file, the
	// lines, the first of them, and a way to drop the quote.
	type Props = { file: string; range: string; snippet: string; onremove: () => void };
	let { file, range, snippet, onremove }: Props = $props();
</script>

<div class="flex w-full items-center gap-2 rounded-sm border-l-2 border-border bg-surface-2 px-2 py-1.5 font-mono text-caption">
	<span class="inline-flex shrink-0 text-text-2"><Icon icon={FileCode} size={13} /></span>
	<span class="shrink-0 text-text">{m.code_quote_ref({ file, range })}</span>
	<span class="min-w-0 flex-1 truncate text-text-3">{snippet}</span>
	<button type="button" aria-label={m.code_quote_remove()} class="inline-flex shrink-0 cursor-pointer rounded-xs text-text-3 outline-offset-1 hover:text-text pointer-coarse:size-11 pointer-coarse:items-center pointer-coarse:justify-center" onclick={onremove}>
		<Icon icon={X} size={12} />
	</button>
</div>
