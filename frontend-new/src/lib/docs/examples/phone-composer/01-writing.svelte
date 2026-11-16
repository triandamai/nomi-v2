<!-- Writing: tap the field for the toolbar; type "@" for the crew picker. -->
<script lang="ts">
	import PhoneComposer from '$lib/ds/thread/PhoneComposer.svelte';

	let sent = $state<string[]>([]);
	const crew = [
		{ kind: 'crew' as const, agent: 'dana' as const, name: 'Dana', role: 'Money and subscriptions' },
		{ kind: 'crew' as const, agent: 'kala' as const, name: 'Kala', role: 'Reminders on time' },
		{ kind: 'crew' as const, agent: 'maya' as const, name: 'Maya', role: 'Reads attachments' },
	];
</script>

<div class="flex w-97.5 max-w-full flex-col gap-3 pt-48">
	{#each sent as text, at (at)}<p class="self-end rounded-3xl bg-surface-2 px-3.5 py-2.5 text-phone text-text">{text}</p>{/each}
	<PhoneComposer placeholder="Message Nomi" {crew} model="deepseek-flash" onsend={(text) => (sent = [...sent, text])} onmodel={() => {}} oncrew={() => {}} />
</div>
