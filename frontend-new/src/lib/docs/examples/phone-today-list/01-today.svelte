<!-- Today: what came in, today's reminders, plans in progress. -->
<script lang="ts">
	import { AlarmClock, Inbox, ListChecks } from '@lucide/svelte';
	import PhoneTodayList from '$lib/ds/thread/PhoneTodayList.svelte';
</script>

<div class="w-97.5 max-w-full">
	<PhoneTodayList
		label="Today"
		rows={[
			{ icon: Inbox, title: 'While you were out', detail: 'A deploy failed · 2 updates', count: 3, href: '#' },
			{ icon: AlarmClock, title: 'Reminders today', detail: 'Stretch 16:00 · Call Siti 20:00', count: 2, href: '#' },
			{ icon: ListChecks, title: 'Plans in progress', detail: 'Bandung weekend · 2 of 5 done', count: 1, href: '#' },
		]}
	/>
</div>
