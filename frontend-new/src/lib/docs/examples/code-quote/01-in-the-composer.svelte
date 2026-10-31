<!-- In the composer: the lines picked in the code view, with a way to drop them. -->
<script lang="ts">
	import CodeQuote from '$lib/ds/build/CodeQuote.svelte';
</script>

<div class="w-full max-w-110">
	<CodeQuote file="+page.svelte" range="14–15" snippet={'<PickupSlots from="07:00" to="11:00" …'} onremove={() => {}} />
</div>
