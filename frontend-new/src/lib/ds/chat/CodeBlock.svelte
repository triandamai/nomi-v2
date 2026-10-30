<script lang="ts">
	import { Check, Copy } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import { createCopier } from '../actions/copier.svelte';
	import CodeToken from './CodeToken.svelte';
	import { plainLines, type CodeLine } from './code';

	// Chat responses → Code: mono, numbered, coloured with the nm-code-* tokens (comments recede
	// to text-3). Copy takes the code alone, never the line numbers.
	type Props = {
		code: string;
		language?: string;
		file?: string;
		/** The highlighted lines; without them the code shows plain. */
		tokens?: CodeLine[];
	};
	let { code, language, file, tokens }: Props = $props();

	const lines = $derived(tokens ?? plainLines(code));
	const copier = createCopier();
</script>

<figure class="flex w-full flex-col overflow-hidden rounded-md bg-bg outline -outline-offset-1 outline-border-strong">
	<figcaption class="flex items-center gap-2.5 border-b border-border bg-surface px-3.5 pt-2 pb-1.75">
		{#if language}<span class="font-mono text-label text-text-3">{language}</span>{/if}
		<span class="min-w-0 flex-1 truncate font-mono text-label text-text-2">{file}</span>
		<button
			type="button"
			class="inline-flex cursor-pointer items-center gap-1.5 rounded-xs text-caption text-text-2 outline-offset-1 hover:text-text pointer-coarse:min-h-11"
			onclick={() => copier.copy(code)}
		>
			<span class="inline-flex text-text-3"><Icon icon={copier.copied ? Check : Copy} size={12} /></span>{copier.copied ? m.copy_done() : m.code_copy()}
		</button>
	</figcaption>
	<pre class="overflow-x-auto py-3 pr-3.5 font-mono text-caption leading-code text-text"><code class="flex min-w-max flex-col">{#each lines as line, index (index)}<span class="flex"><span data-line-number aria-hidden="true" class="w-15 shrink-0 pr-4 text-right text-text-3 select-none">{index + 1}</span><span class="whitespace-pre">{#each line as token, part (part)}<CodeToken {token} />{/each}</span></span
				>{/each}</code></pre>
</figure>
