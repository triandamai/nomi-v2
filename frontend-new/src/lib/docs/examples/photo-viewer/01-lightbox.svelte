<!-- A photo large, with zoom, Download and Close. It opens as a modal: press the button. -->
<script lang="ts">
	import Button from '$lib/ds/actions/Button.svelte';
	import PhotoViewer from '$lib/ds/thread/PhotoViewer.svelte';

	let open = $state(false);
	const receipt = 'data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="460" height="600"><rect width="460" height="600" fill="%23d9d4c7"/></svg>';
</script>

<Button label="Open the photo" variant="secondary" onclick={() => (open = true)} />
{#if open}
	<PhotoViewer name="struk-belanja-okt.jpg" meta="JPG · 1,8 MB · your attachment · sent 11:20" src={receipt} where="1 of 1 photo in this message" downloadHref="#" onclose={() => (open = false)} />
{/if}
