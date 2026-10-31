<!-- Interrupted: cut off, with the provider's reason and the next move. -->
<script lang="ts">
	import ReplyText from '$lib/ds/chat/ReplyText.svelte';
</script>

<div class="w-full max-w-170">
	<ReplyText
		text="Food is October's biggest jump: Rp 2,1 jt against Rp 1,4 jt in September. Most of the difference is"
		status="interrupted"
		reason="Reply cut off after 18s. DeepSeek closed the stream."
		onrestart={() => {}}
		oncontinue={() => {}}
	/>
</div>
