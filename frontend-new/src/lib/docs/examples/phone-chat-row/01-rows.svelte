<!-- Rows: who answered where it's known, the glyph where it isn't, one the crew is working in. -->
<script lang="ts">
	import PhoneChatRow from '$lib/ds/thread/PhoneChatRow.svelte';
</script>

<div class="flex w-97.5 max-w-full flex-col">
	<PhoneChatRow href="#chat-1" agent="dana" title="Food spending in October" time="09:41" preview="Rp 2,1 jt on food, up 20% from September" />
	<PhoneChatRow href="#chat-2" title="Weekly groceries" time="17:02" preview="Your usual Sayurbox list is ready" />
	<PhoneChatRow href="#chat-3" title="Bandung weekend" working="working" preview="Sharing the itinerary with Sari" />
</div>
