<!-- Kinds: connection, provider, key and plan; one at a time above the thread. -->
<script lang="ts">
	import Banner from '$lib/ds/chat/Banner.svelte';
</script>

<div class="flex w-full max-w-170 flex-col gap-5">
	<Banner kind="offline" text="You're offline. New messages wait in the outbox and send when you reconnect." action={{ label: 'Retry now', onclick: () => {} }} />
	<Banner kind="online" text="Back online. 1 queued message sent." />
	<Banner kind="slow" text="DeepSeek is slow right now (replies ~14s). Your chats still work." />
	<Banner kind="expiring" text="Your DeepSeek key expires in 3 days." action={{ label: 'Update key', onclick: () => {} }} />
	<Banner kind="quota" text="82% of this month's allowance used · resets 1 Nov." action={{ label: 'See usage', onclick: () => {} }} />
</div>
