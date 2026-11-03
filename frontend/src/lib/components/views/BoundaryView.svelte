<!-- Something the crew won't or can't do (Chat responses → Boundary): the reason said plainly and
     what it can do instead, each owned by the crew member who'd do it. Picking one sends it as the
     person's message, or opens its page. -->
<script lang="ts">
	import { useChatSend } from '$lib/chatSend';
	import { crewId } from '$lib/crewId';
	import Boundary from '$lib/components/ds/chat/Boundary.svelte';
	import type { BoundaryView, ViewProps } from '$lib/views/types';

	let { view, agent = null, answered = false }: ViewProps<BoundaryView> = $props();

	const send = useChatSend();

	const alternatives = $derived(
		(view.alternatives ?? []).map((alternative) => {
			const external = !!alternative.url && /^https?:\/\//i.test(alternative.url);
			return {
				agent: crewId(alternative.agent ?? agent),
				title: alternative.title,
				detail: alternative.detail,
				href: alternative.url,
				external,
				onclick: alternative.message && send && !answered ? () => send(alternative.message!) : undefined,
			};
		}),
	);
</script>

<Boundary title={view.title} reason={view.reason} {alternatives} />
