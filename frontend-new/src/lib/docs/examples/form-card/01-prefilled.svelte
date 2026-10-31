<!-- Prefilled: a real form in the thread, filled in from the chat. -->
<script lang="ts">
	import FormCard from '$lib/ds/chat/FormCard.svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';
</script>

<div class="w-full max-w-170">
	<FormCard
		agent="kala"
		title="New reminder"
		meta="prefilled from chat"
		note="Kala pings you in Inbox and on your phone"
		submitLabel="Save reminder"
		onsubmit={() => {}}
		oncancel={() => {}}
	>
		<TextField label="What" value="Pay kos rent to Bu Sri" />
		<div class="grid gap-3 sm:grid-cols-2">
			<TextField label="Date" value="25 Nov 2026" mono />
			<TextField label="Time" value="09:00" mono />
		</div>
	</FormCard>
</div>
