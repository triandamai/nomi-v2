<!-- Not sent yet: faded, and saying where it is, with the next move. -->
<script lang="ts">
	import MessageUser from '$lib/ds/conversation/MessageUser.svelte';
</script>

<div class="flex w-full max-w-180 flex-col gap-4">
	<MessageUser text="hey" status="sending" />
	<MessageUser text="hey" status="queued" oncancel={() => {}} />
	<MessageUser text="hey" status="failed" onretry={() => {}} onedit={() => {}} />
</div>
