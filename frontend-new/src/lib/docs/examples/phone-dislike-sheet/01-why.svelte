<!-- Dislike: what went wrong, and a note. -->
<script lang="ts">
	import PhoneDislikeSheet from '$lib/ds/thread/PhoneDislikeSheet.svelte';
	import Button from '$lib/ds/actions/Button.svelte';

	let open = $state(false);
</script>

<Button variant="secondary" label="Bad reply" onclick={() => (open = true)} />
<PhoneDislikeSheet bind:open onsend={() => {}} />
