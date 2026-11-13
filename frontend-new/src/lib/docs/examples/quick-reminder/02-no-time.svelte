<!-- No time yet: the day is read, the time is asked for. -->
<script lang="ts">
	import { Calendar } from '@lucide/svelte';
	import QuickReminder from '$lib/ds/inbox/QuickReminder.svelte';

	let value = $state('water the plants tomorrow');
</script>

<div class="w-full max-w-75">
	<QuickReminder label="Quick reminder" bind:value chips={[{ icon: Calendar, label: 'Thu 22 Oct' }]} hint="add a time" />
</div>
