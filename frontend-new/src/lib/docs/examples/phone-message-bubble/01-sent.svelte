<!-- Sent and not sent: your message on the right, at 16pt; one that didn't go says so with Retry. -->
<script lang="ts">
	import PhoneMessageBubble from '$lib/ds/thread/PhoneMessageBubble.svelte';
</script>

<div class="flex w-89.5 max-w-full flex-col gap-6">
	<PhoneMessageBubble text="How much did I spend on food in October, without the office lunches?" />
	<PhoneMessageBubble text="And in September?" status="failed" onretry={() => {}} />
</div>
