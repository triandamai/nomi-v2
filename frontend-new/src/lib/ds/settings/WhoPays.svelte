<script lang="ts">
	// App / Settings / Models & providers (RbTgo) → Who Pays: what each kind of provider costs, its
	// tag (Included in the accent, Your key and Local quiet) over a line about it, 12 apart.
	type Props = { label: string; kinds: { tag: string; text: string; accent?: boolean }[] };
	let { label, kinds }: Props = $props();
</script>

<section aria-label={label} class="flex flex-col gap-3">
	<p class="font-mono text-label tracking-label text-text-3 uppercase">{label}</p>
	<dl class="flex flex-col gap-3">
		{#each kinds as kind (kind.tag)}
			<div class="flex flex-col items-start gap-1">
				<dt class={['inline-flex h-5 items-center rounded-xs px-1.5 font-mono text-label tracking-label uppercase', kind.accent ? 'bg-accent-bg text-accent' : 'bg-surface-2 text-text-2']}>{kind.tag}</dt>
				<dd class="text-caption text-text-3">{kind.text}</dd>
			</div>
		{/each}
	</dl>
</section>
