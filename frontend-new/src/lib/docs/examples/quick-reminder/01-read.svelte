<!-- Read: what the line says, as chips, before it’s set. -->
<script lang="ts">
	import { Calendar, Clock, Repeat } from '@lucide/svelte';
	import QuickReminder from '$lib/ds/inbox/QuickReminder.svelte';

	let value = $state('pay PLN electricity every 25th at 10am');
</script>

<div class="w-full max-w-75">
	<QuickReminder
		label="Quick reminder"
		placeholder="Pay rent every 25th at 10am"
		bind:value
		chips={[
			{ icon: Calendar, label: 'Sun 25 Oct' },
			{ icon: Clock, label: '10:00' },
			{ icon: Repeat, label: 'monthly' },
		]}
		hint="↵ add"
	/>
</div>
