<script lang="ts">
	import { ArrowUp, Eye } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import PeerAvatar from '../build/PeerAvatar.svelte';
	import PhoneButton from './PhoneButton.svelte';

	// Mobile / Publish Request: a teammate's change is ready and needs someone who can publish.
	// Publish it, or look at it in the preview first.
	type Props = { who: { name: string; initials: string; peer?: 1 | 2 }; meta: string; onpublish: () => void; onreview: () => void };
	let { who, meta, onpublish, onreview }: Props = $props();
</script>

<section class="flex w-full flex-col gap-3 rounded-3xl border border-border bg-surface p-3.75">
	<div class="flex items-center gap-2.5">
		<PeerAvatar initials={who.initials} peer={who.peer} />
		<p class="flex min-w-0 flex-1 flex-col gap-0.5">
			<span class="text-body font-medium text-text">{m.change_ready({ name: who.name.split(/\s+/)[0] })}</span>
			<span class="font-mono text-label text-text-3">{meta}</span>
		</p>
	</div>
	<PhoneButton label={m.ws_publish()} icon={ArrowUp} onclick={onpublish} />
	<PhoneButton label={m.publish_review()} variant="secondary" icon={Eye} onclick={onreview} />
</section>
