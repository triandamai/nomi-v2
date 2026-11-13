<script lang="ts">
	// App / Settings / Memory (hANTE) → Strength: how strong a memory is, five 12 × 4 bars 2 apart
	// (as many filled as its level, in the text colour) over its name in mono. Faint is the warning:
	// one bar and the name in the warning colour.
	type Props = { level: 1 | 2 | 3 | 4 | 5; name: string };
	let { level, name }: Props = $props();
</script>

<span class="flex flex-col gap-1" role="img" aria-label={name}>
	<span class="flex gap-0.5" aria-hidden="true">
		{#each [1, 2, 3, 4, 5] as step (step)}
			<span class={['h-1 w-3 rounded-hair', step > level ? 'bg-surface-2' : level === 1 ? 'bg-warning' : 'bg-text']}></span>
		{/each}
	</span>
	<span aria-hidden="true" class={['font-mono text-label', level === 1 ? 'text-warning' : 'text-text-3']}>{name}</span>
</span>
