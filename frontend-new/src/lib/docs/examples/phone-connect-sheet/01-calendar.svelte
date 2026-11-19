<!-- Tara needs Google Calendar: Connect signs in and comes back to the chat. -->
<script lang="ts">
	import PhoneConnectSheet from '$lib/ds/thread/PhoneConnectSheet.svelte';

	let open = $state(false);
</script>

<button type="button" class="rounded-sm border border-border-strong px-3 py-1.5 text-ui text-text" onclick={() => (open = true)}>Connect Google Calendar</button>
<PhoneConnectSheet bind:open agent="tara" agentName="Tara" service="Calendar" provider="google" services={['calendar']} resume="00000000-0000-0000-0000-000000000000" onanswer={() => {}} />
