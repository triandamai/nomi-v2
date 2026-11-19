<!-- New chat: the pen's starters, each to the crew member who owns it. -->
<script lang="ts">
	import PhoneStarters from '$lib/ds/thread/PhoneStarters.svelte';
</script>

<div class="w-89.5 max-w-full">
	<PhoneStarters
		items={[
			{ agent: 'koda', label: 'Build a menu page for my warung', route: '→ koda' },
			{ agent: 'maya', label: 'Turn a photo of my price list into an order form', route: '→ maya' },
			{ agent: 'nomi', label: 'What can I build in 10 minutes?', route: '→ nomi' },
			{ agent: 'rena', label: 'Remind me to check pre-orders at 4pm', route: '→ rena' },
		]}
		onpick={() => {}}
	/>
</div>
