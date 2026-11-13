<!-- In a chat: the model and the thinking level; type @ for the crew. -->
<script lang="ts">
	import ChatComposer from '$lib/ds/thread/ChatComposer.svelte';

	let thinking = $state('medium');
	const levels = [
		{ value: 'low', label: 'Quick' },
		{ value: 'medium', label: 'Default' },
		{ value: 'high', label: 'Deep' },
	];
</script>

<div class="w-full max-w-180 pt-40">
	<ChatComposer
		placeholder="Ask anything…"
		pickerAbove
		crew={[
			{ kind: 'crew', agent: 'dana', name: 'Dana', role: 'Money and subscriptions' },
			{ kind: 'crew', agent: 'kala', name: 'Kala', role: 'Reminders on time' },
		]}
		model="deepseek-flash"
		models={[{ group: 'Nomi credits', models: [{ key: 'admin:1', label: 'deepseek-flash', source: 'Nomi credits', chosen: true }] }]}
		thinking={{ label: levels.find((level) => level.value === thinking)?.label ?? '', value: thinking, levels }}
		onthinking={(value) => (thinking = value)}
		onsend={() => {}}
	/>
</div>
