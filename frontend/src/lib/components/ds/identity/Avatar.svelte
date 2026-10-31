<script lang="ts">
	// A person: their picture, or their initials.
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
	<img data-avatar {src} alt={name} class="avatar" />
{:else}
	<span data-avatar role="img" aria-label={name} class="avatar avatar--initials">
		<span aria-hidden="true">{initials}</span>
	</span>
{/if}

<style>
	.avatar {
		width: 32px;
		height: 32px;
		flex-shrink: 0;
		border-radius: var(--md-sys-shape-corner-full);
		object-fit: cover;
	}
	.avatar--initials {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
		font: var(--md-sys-typescale-label-medium-weight) var(--md-sys-typescale-label-medium-size) / 1 var(--md-sys-typescale-label-medium-font);
	}
</style>
