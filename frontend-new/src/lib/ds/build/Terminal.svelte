<script lang="ts">
	import { ArrowRight, Check, CircleAlert, Copy, Info, SquareTerminal } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';
	import { createCopier } from '../actions/copier.svelte';

	// Build → Terminal: what Koda ran, in order, read-only, with the time of each command. A failure
	// sums up in one line at the foot, with what went live and a way to ask for the fix.
	type Line = { kind: 'command' | 'out' | 'ok' | 'fail' | 'error' | 'warn' | 'blank'; text?: string; time?: string };
	type Props = {
		/** e.g. "run #2 · publish deploy #2". */
		run: string;
		lines: Line[];
		failure?: { title: string; detail?: string; onfix: () => void };
	};
	let { run, lines, failure }: Props = $props();

	const copier = createCopier();
	const log = $derived(lines.map((line) => line.text ?? '').join('\n'));
</script>

<section class="flex h-130 w-full flex-col overflow-hidden rounded-sm border border-border-strong bg-bg">
	<header class="flex h-10 shrink-0 items-center gap-2.5 border-b border-border px-4">
		<span class="inline-flex text-text"><Icon icon={SquareTerminal} size={14} /></span>
		<span class="truncate font-mono text-caption text-text">{run}</span>
		<span class="min-w-0 flex-1 truncate font-mono text-caption text-text-3">{m.term_meta({ name: 'Koda' })}</span>
		<button type="button" class="inline-flex h-6.5 shrink-0 cursor-pointer items-center gap-1.5 rounded-sm border border-border-strong px-2.5 text-caption text-text outline-offset-0 hover:bg-surface-2 focus-visible:outline-3 pointer-coarse:min-h-11" onclick={() => copier.copy(log)}>
			<Icon icon={copier.copied ? Check : Copy} size={14} />{copier.copied ? m.copy_done() : m.term_copy()}
		</button>
	</header>
	<div role="log" aria-label={run} class="flex min-h-0 flex-1 flex-col gap-1 overflow-auto px-4 py-3.5 font-mono text-caption">
		{#each lines as line, index (index)}
			<p class={['flex min-h-5 items-center gap-3', (line.kind === 'fail') && 'bg-danger-bg']}>
				<span
					class={[
						'min-w-0 flex-1 whitespace-pre-wrap',
						line.kind === 'command' && 'font-medium text-text',
						line.kind === 'out' && 'text-text-2',
						line.kind === 'ok' && 'text-accent',
						(line.kind === 'fail' || line.kind === 'error') && 'text-danger',
						line.kind === 'warn' && 'text-warning',
					]}>{line.text ?? ''}</span
				>
				{#if line.time}<span class="text-text-3">{line.time}</span>{/if}
			</p>
		{/each}
	</div>
	{#if failure}
		<footer class="flex shrink-0 flex-wrap items-center gap-3 border-t border-border bg-surface px-4 py-3">
			<span class="inline-flex text-danger"><Icon icon={CircleAlert} size={14} /></span>
			<span class="flex min-w-0 flex-1 basis-48 flex-col gap-0.5">
				<span class="text-ui font-medium text-text">{failure.title}</span>
				{#if failure.detail}<span class="text-caption text-text-2">{failure.detail}</span>{/if}
			</span>
			<Button label={m.term_fix({ name: 'Koda' })} iconEnd={ArrowRight} onclick={failure.onfix} />
		</footer>
	{:else}
		<footer class="flex h-8.5 shrink-0 items-center gap-2 border-t border-border px-4 text-caption text-text-3">
			<Icon icon={Info} size={12} />{m.term_note({ name: 'Koda' })}
		</footer>
	{/if}
</section>
