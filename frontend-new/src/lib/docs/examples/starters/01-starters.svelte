<!-- Starters: each routed to who owns it. -->
<script lang="ts">
	import Starters from '$lib/ds/thread/Starters.svelte';
</script>

<div class="w-full max-w-208">
	<Starters
		items={[
			{ agent: 'dana', label: 'Summarize this week’s spending', route: '→ dana' },
			{ agent: 'rena', label: 'Plan my Saturday', route: '→ rena' },
			{ agent: 'koda', label: 'Start a small website project', route: '→ koda' },
			{ agent: 'kala', label: 'Remind me to stretch at 4pm', route: '→ kala' },
		]}
		onpick={() => {}}
	/>
</div>
