<script lang="ts">
	// Flow / Build — Workspace tools (U16wU) → Screen / Monitor (EQNtT) → Logs: what the running app
	// prints, its newest lines in mono (an error's line in the danger colour, a warning's in the
	// warning colour), scrolled within.
	type Props = { label: string; lines: string[] };
	let { label, lines }: Props = $props();

	const tone = (line: string) => (/\b(error|fatal|panic)\b/i.test(line) ? 'error' : /\bwarn(ing)?\b/i.test(line) ? 'warn' : 'info');
</script>

<div role="log" aria-label={label} class="flex max-h-60 flex-col gap-0.5 overflow-y-auto font-mono text-label">
	{#each lines as line, index (index)}
		<span class={['py-1.25 break-all whitespace-pre-wrap', tone(line) === 'error' ? 'text-danger' : tone(line) === 'warn' ? 'text-warning' : 'text-text-2']}>{line}</span>
	{/each}
</div>
