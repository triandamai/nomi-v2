<script lang="ts">
	// Design System / Nomi → Identity: a person. Their picture, or their initials in mono.
	let { name, src }: { name: string; src?: string } = $props();

	const initials = $derived(
		name
			.trim()
			.split(/\s+/)
			.slice(0, 2)
			.map((part) => part[0]?.toUpperCase() ?? '')
			.join(''),
	);
</script>

{#if src}
	<img data-avatar {src} alt={name} class="size-7 shrink-0 rounded-sm border border-border-strong object-cover" />
{:else}
	<span data-avatar role="img" aria-label={name} class="inline-flex size-7 shrink-0 items-center justify-center rounded-sm border border-border-strong bg-surface-2">
		<span aria-hidden="true" class="font-mono text-label text-text-2">{initials}</span>
	</span>
{/if}
