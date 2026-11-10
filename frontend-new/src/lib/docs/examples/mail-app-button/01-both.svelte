<!-- Both: side by side, sharing the row. Offer only the one that serves the address. -->
<script lang="ts">
	import MailAppButton from '$lib/ds/auth/MailAppButton.svelte';
</script>

<div class="flex w-full max-w-95 gap-2">
	<MailAppButton app="gmail" />
	<MailAppButton app="outlook" />
</div>
