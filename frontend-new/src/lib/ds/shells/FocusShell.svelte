<script lang="ts">
	import type { Snippet } from 'svelte';

	// Layout shells → Focus shell (onboarding, dialogs, checkout): one centred column, 480 or 720.
	let { width = 'narrow', children }: { width?: 'narrow' | 'wide'; children?: Snippet } = $props();
</script>

<div class="flex min-h-dvh w-full justify-center bg-bg px-4 py-12 text-text">
	<div data-column class={['w-full', width === 'narrow' ? 'max-w-120' : 'max-w-180']}>{@render children?.()}</div>
</div>
