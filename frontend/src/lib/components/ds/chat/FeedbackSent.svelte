<script lang="ts">
	import { Check } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Chat responses → Feedback / Sent: thanks, and exactly what went with it. Offers another try,
	// e.g. with a different model.
	type Props = { withChat?: boolean; retry?: { label: string; onclick: () => void } };
	let { withChat = false, retry }: Props = $props();
</script>

<p role="status" class="flex w-full flex-wrap items-center gap-x-2 gap-y-1 rounded-sm bg-surface px-3 py-2">
	<span class="inline-flex shrink-0 text-accent"><Icon icon={Check} size={14} /></span>
	<span class="min-w-0 flex-1 basis-40 text-caption text-text-2">{withChat ? m.feedback_sent_chat() : m.feedback_sent()}</span>
	{#if retry}
		<button type="button" class="ml-auto cursor-pointer rounded-xs text-caption text-text outline-offset-1 hover:underline hover:decoration-border-strong hover:underline-offset-2 pointer-coarse:min-h-11" onclick={retry.onclick}>{retry.label}</button>
	{/if}
</p>
