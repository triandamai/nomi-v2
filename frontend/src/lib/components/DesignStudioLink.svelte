<script lang="ts">
	import { Palette } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import Icon from '$lib/components/ds/foundations/Icon.svelte';

	// The way into a chat's Design Studio, in the chat's top bar, once Sora has made a preview there.
	let { sessionId, versions }: { sessionId: string; versions: number } = $props();
</script>

{#if versions > 0}
	<IconButton href={`/chat/${sessionId}/design`} aria-label={m.chat_open_design_studio()} title={m.chat_open_design_studio()}>
		<Icon icon={Palette} size={18} />
	</IconButton>
{/if}
