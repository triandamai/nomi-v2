<!-- Scheduled and due: when and who keeps it; due, it offers snooze and done. -->
<script lang="ts">
	import Reminder from '$lib/ds/chat/Reminder.svelte';
</script>

<div class="flex w-full max-w-170 flex-col gap-5">
	<Reminder title="Pay kos rent to Bu Sri" when="25 Nov · 09:00 · every month" owner={{ agent: 'kala', name: 'Kala' }} onedit={() => {}} />
	<Reminder title="Pay kos rent to Bu Sri" when="25 Nov 09:00 · every month" due doneLabel="Mark paid" onsnooze={() => {}} ondone={() => {}} />
</div>
