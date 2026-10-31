<!-- Reasons, then an optional note: only this reply goes with it unless you tick the whole chat. -->
<script lang="ts">
	import DislikeFeedback from '$lib/ds/chat/DislikeFeedback.svelte';
</script>

<div class="w-full max-w-170">
	<DislikeFeedback
		initialReasons={['inaccurate', 'wrong_crew']}
		initialNote="It counted the Rp 300rb of office lunches my work pays back. Those aren't my spending."
		onsend={() => {}}
		oncancel={() => {}}
	/>
</div>
