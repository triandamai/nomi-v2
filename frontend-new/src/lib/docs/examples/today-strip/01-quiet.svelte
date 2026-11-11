<!-- A quiet day: nothing yet, and what to do about it. -->
<script lang="ts">
	import TodayStrip from '$lib/ds/thread/TodayStrip.svelte';
</script>

<div class="w-full max-w-208">
	<TodayStrip
		updated="08:14"
		columns={[
			{ key: 'out', title: 'While you were out', count: 0, body: 'All quiet. Nothing new from the crew since you last looked.', action: { label: 'Open notifications', href: '#inbox' } },
			{ key: 'reminders', title: 'Reminders', count: 0, body: 'Nothing scheduled today. Ask Kala, or add one yourself.', action: { label: 'Add reminder', onclick: () => {} } },
			{ key: 'plans', title: 'Plans in progress', count: 0, body: 'No plans running. Ask Rena to draft one and progress shows up here.', action: { label: 'Start a plan', onclick: () => {} } },
		]}
	/>
</div>
