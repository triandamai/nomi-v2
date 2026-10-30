<script lang="ts">
	import { ArrowRight, Users } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Button from '../actions/Button.svelte';
	import PeerAvatar from './PeerAvatar.svelte';

	// Build together → Publish request: a teammate who can prompt but not publish has a change
	// built and tested in the preview; the publisher decides.
	type Props = { who: { name: string; initials: string; peer?: 1 | 2 }; meta?: string; onpublish: () => void; onreview?: () => void };
	let { who, meta, onpublish, onreview }: Props = $props();
</script>

<section class="flex w-full flex-col overflow-hidden rounded-md border border-border bg-surface">
	<header class="flex items-center gap-2.5 px-3.5 py-3">
		<PeerAvatar initials={who.initials} peer={who.peer} />
		<span class="flex min-w-0 flex-1 flex-col gap-0.5">
			<span class="text-ui font-medium text-text">{m.publish_request_title({ name: who.name.split(/\s+/)[0] })}</span>
			{#if meta}<span class="font-mono text-label text-text-3">{meta}</span>{/if}
		</span>
	</header>
	<footer class="flex flex-wrap gap-2 border-t border-border px-3.5 py-2.5">
		<Button label={m.ws_publish()} iconEnd={ArrowRight} onclick={onpublish} />
		{#if onreview}<Button label={m.publish_review()} variant="ghost" iconStart={Users} onclick={onreview} />{/if}
	</footer>
</section>
