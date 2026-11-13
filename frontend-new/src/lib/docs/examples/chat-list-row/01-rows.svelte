<!-- Rows: a pinned chat, one the crew is working in, and one from earlier. -->
<script lang="ts">
	import ChatListRow from '$lib/ds/thread/ChatListRow.svelte';

	let pinned = $state(true);
</script>

<div class="flex w-full max-w-180 flex-col gap-0.5">
	<ChatListRow href="#bandung" title="Bandung weekend" preview="Tara: Share the itinerary with Sari?" time="10:08" {pinned} onpin={() => (pinned = !pinned)} ondelete={() => {}} />
	<ChatListRow href="#food" title="Food spending in October" preview="Rp 2,1 jt on food, up 20% from September" time="09:41" working onpin={() => {}} ondelete={() => {}} />
	<ChatListRow href="#rent" title="Kos rent tracker" preview="Saved. I'll ping you on the 25th." time="Mon 19 Oct" onpin={() => {}} ondelete={() => {}} />
</div>
