<!-- Signed out: the draft kept, one way on (it can't be dismissed, so here it closes after 5 seconds). -->
<script lang="ts">
	import PhoneSignedOutSheet from '$lib/ds/thread/PhoneSignedOutSheet.svelte';
	import Button from '$lib/ds/actions/Button.svelte';

	let open = $state(false);
	function show() {
		open = true;
		setTimeout(() => (open = false), 5000);
	}
</script>

<Button variant="secondary" label="Show the sheet for 5 seconds" onclick={show} />
<PhoneSignedOutSheet {open} draft="Find Whoosh trains for Sat 1 Nov…" signInHref="#sign-in" />
