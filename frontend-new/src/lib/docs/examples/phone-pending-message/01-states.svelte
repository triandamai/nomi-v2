<!-- Queued, and not sent. -->
<script lang="ts">
	import PhonePendingMessage from '$lib/ds/thread/PhonePendingMessage.svelte';
</script>

<div class="flex w-89.5 flex-col gap-6">
	<PhonePendingMessage text="How much did I spend on food in October? @Dana" status="queued" note="queued · sends when you’re back" action={{ label: 'Cancel', onclick: () => {} }} />
	<PhonePendingMessage text="Read the January to October statements" status="failed" note="not sent · Nomi server error 500" action={{ label: 'Retry', onclick: () => {} }} />
</div>
