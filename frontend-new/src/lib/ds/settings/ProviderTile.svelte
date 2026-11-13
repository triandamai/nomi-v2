<script lang="ts">
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// App / Settings / Models & providers (RbTgo) → Logo: a provider's mark on surface-2, 32 in its
	// card and 20 in the Connect list. Nomi's own models wear Nomi's mascot; a provider shows its
	// initial (v2 doesn't carry the providers' logos yet).
	type Props = { name: string; mascot?: CrewId; size?: 32 | 20 };
	let { name, mascot, size = 32 }: Props = $props();
</script>

<span
	aria-hidden="true"
	class={[
		'inline-flex shrink-0 items-center justify-center font-mono text-text-2',
		size === 32 ? 'size-8 rounded-sm bg-surface-2 text-ui' : 'size-5 rounded-xs bg-surface text-label outline -outline-offset-1 outline-border',
	]}
>
	{#if mascot}<Mascot agent={mascot} size={20} />{:else}{name.charAt(0).toUpperCase()}{/if}
</span>
