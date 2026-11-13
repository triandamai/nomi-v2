<!-- While the crew answers: Send becomes Stop; hover it for the hint. -->
<script lang="ts">
	import ChatComposer from '$lib/ds/thread/ChatComposer.svelte';
</script>

<div class="w-full max-w-180 pt-10">
	<ChatComposer placeholder="Ask anything…" busy onstop={() => {}} onsend={() => {}} />
</div>
