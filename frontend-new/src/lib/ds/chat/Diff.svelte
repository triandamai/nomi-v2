<script lang="ts">
	import { Check, FileDiff } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';
	import { diffStats, type DiffHunk } from './code';

	// Chat responses → Diff: a crew member's edits to one file, applied only on approval. Old and new
	// line numbers, the sign and a tinted background for each added or removed line.
	type Props = {
		path: string;
		hunks: DiffHunk[];
		/** Who asks to make the change, and how many files the request touches. */
		request?: { agent: CrewId; name: string; files: number };
		onapply?: () => void;
		onreject?: () => void;
	};
	let { path, hunks, request, onapply, onreject }: Props = $props();

	const stats = $derived(diffStats(hunks));
</script>

<figure class="flex w-full flex-col overflow-hidden rounded-md border border-border-strong bg-bg">
	<figcaption class="flex items-center gap-2.5 border-b border-border bg-surface px-3.5 py-2">
		<span class="inline-flex text-text-2"><Icon icon={FileDiff} size={14} /></span>
		<span class="min-w-0 flex-1 truncate font-mono text-label text-text-2">{path}</span>
		<span class="sr-only">{m.diff_stats({ added: String(stats.added), removed: String(stats.removed) })}</span>
		<span aria-hidden="true" class="font-mono text-label text-accent">+{stats.added}</span>
		<span aria-hidden="true" class="font-mono text-label text-danger">−{stats.removed}</span>
	</figcaption>
	<pre class="overflow-x-auto pb-1.5 font-mono text-caption leading-code"><code class="flex min-w-max flex-col">{#each hunks as hunk, hunkIndex (hunkIndex)}<span class="bg-surface px-3.5 py-1 text-label text-text-3">{hunk.header}</span>{#each hunk.lines as line, index (index)}<span
						data-kind={line.kind}
						class={['flex', line.kind === 'add' && 'bg-diff-add', line.kind === 'del' && 'bg-diff-del']}
						><span aria-hidden="true" class="w-9 shrink-0 text-right text-text-3 select-none">{line.old ?? ''}</span><span aria-hidden="true" class="w-9 shrink-0 text-right text-text-3 select-none">{line.new ?? ''}</span><span
							aria-hidden="true"
							class={['w-6 shrink-0 text-center select-none', line.kind === 'add' && 'text-accent', line.kind === 'del' && 'text-danger']}>{line.kind === 'add' ? '+' : line.kind === 'del' ? '−' : ''}</span
						>{#if line.kind !== 'context'}<span class="sr-only">{line.kind === 'add' ? m.diff_added() : m.diff_removed()}: </span>{/if}<span
							class={['pr-3.5 whitespace-pre', line.kind === 'context' ? 'text-text-2' : 'text-text']}>{line.text}</span
						></span
					>{/each}{/each}</code></pre>
	{#if request || onapply || onreject}
		<footer class="flex flex-wrap items-center gap-2 border-t border-border px-3.5 py-2.5">
			<span class="flex min-w-0 flex-1 basis-48 items-center gap-1.5">
				{#if request}
					<Mascot agent={request.agent} size={20} />
					<span class="text-caption text-text-2">
						{request.files === 1 ? m.diff_request_one({ name: request.name }) : m.diff_request({ name: request.name, count: String(request.files) })}
					</span>
				{/if}
			</span>
			<span class="flex items-center gap-2">
				{#if onreject}<Button label={m.diff_reject()} variant="secondary" onclick={onreject} />{/if}
				{#if onapply}<Button label={m.diff_apply()} iconEnd={Check} onclick={onapply} />{/if}
			</span>
		</footer>
	{/if}
</figure>
