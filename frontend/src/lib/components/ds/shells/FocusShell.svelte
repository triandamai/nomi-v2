<script lang="ts">
	import type { Snippet } from 'svelte';

	// The focus shell (onboarding, dialogs, checkout): one centred column, 480 or 720.
	let { width = 'narrow', children }: { width?: 'narrow' | 'wide'; children?: Snippet } = $props();
</script>

<div class="focus">
	<div data-column class="column" class:column--wide={width === 'wide'}>{@render children?.()}</div>
</div>

<style>
	.focus {
		display: flex;
		min-height: 100dvh;
		width: 100%;
		justify-content: center;
		padding: 48px 16px;
		background: var(--nomi-color-ground);
		color: var(--md-sys-color-on-surface);
	}
	.column {
		width: 100%;
		max-width: 480px;
	}
	.column--wide {
		max-width: 720px;
	}
</style>
