<!-- A page worth opening, as a preview (Chat responses → Link Preview): its site, title,
     description and image (read from the page on the server when the agent didn't give them). The
     whole card opens it in a new tab. -->
<script lang="ts">
	import LinkPreview from '$lib/components/ds/chat/LinkPreview.svelte';
	import type { LinkView, ViewProps } from '$lib/views/types';

	let { view }: ViewProps<LinkView> = $props();

	const host = $derived.by(() => {
		try {
			return new URL(view.url).hostname.replace(/^www\./, '');
		} catch {
			return view.url;
		}
	});
	// Only the page's own picture over https; anything else would be mixed content or a tracker.
	const image = $derived(view.image?.startsWith('https://') ? view.image : undefined);
</script>

<LinkPreview href={view.url} host={view.site ?? host} title={view.title ?? view.url} description={view.description} {image} />
