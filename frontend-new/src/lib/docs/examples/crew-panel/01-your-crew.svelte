<!-- Your crew: how many are idle or working, then each member. One without a mascot shows their initial. -->
<script lang="ts">
	import CrewPanel from '$lib/ds/chrome/CrewPanel.svelte';
</script>

<!-- As in AppShell's panel: 320 wide, its stroke on the left. -->
<div class="w-80 border-l border-border">
	<CrewPanel
		summary={{ kind: 'live', label: '1 working · 3 idle' }}
		members={[
			{ agent: 'nomi', name: 'Nomi', role: 'Talks with you and routes the work' },
			{ agent: 'dana', name: 'Dana', role: 'Transactions, budgets, subscriptions', state: 'working' },
			{ agent: 'kala', name: 'Kala', role: 'Reminds you on time, and keeps your reminders' },
			{ name: 'Iris', role: 'Looks into what went wrong' },
		]}
	/>
</div>
