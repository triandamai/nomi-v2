<script lang="ts">
	let { name, avatarUrl, size = 32 }: { name: string; avatarUrl?: string | null; size?: number } = $props();

	const initials = $derived(
		name
			.trim()
			.split(/\s+/)
			.slice(0, 2)
			.map((part) => part[0]?.toUpperCase() ?? '')
			.join('') || '?',
	);
</script>

{#if avatarUrl}
	<img src={avatarUrl} alt={name} class="m3-avatar" style="width: {size}px; height: {size}px;" />
{:else}
	<div
		class="m3-avatar m3-avatar--initials"
		style="width: {size}px; height: {size}px; font-size: {Math.round(size * 0.4)}px;"
	>
		{initials}
	</div>
{/if}

<style>
	.m3-avatar {
		border-radius: var(--md-sys-shape-corner-full);
		object-fit: cover;
		flex-shrink: 0;
	}
	.m3-avatar--initials {
		display: flex;
		align-items: center;
		justify-content: center;
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
		font-family: var(--md-sys-typescale-label-large-font);
		font-weight: 600;
	}
</style>
