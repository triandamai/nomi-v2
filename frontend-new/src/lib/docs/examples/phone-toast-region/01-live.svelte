<!-- A deploy that went live, at the top of the phone screen: press Show. -->
<script lang="ts">
	import PhoneToastRegion from '$lib/ds/overlays/PhoneToastRegion.svelte';
	import { toasts } from '$lib/ds/overlays/toasts.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
</script>

<Button label="Show" onclick={() => toasts.show({ kind: 'success', title: 'Deploy #2 is live', action: { label: 'Roll back to #1', onclick: () => {} } })} />
<PhoneToastRegion />
