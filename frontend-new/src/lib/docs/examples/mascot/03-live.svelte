<!-- Live: at 96 and 32 the mascot gains volume, blinks, follows the pointer and plays its mood (thinking while it works). Hover one to cheer it up. -->
<script lang="ts">
	import Mascot from '$lib/ds/agents/Mascot.svelte';
	import { CREW } from '$lib/ds/agents/crew';
	import Segmented from '$lib/ds/inputs/Segmented.svelte';

	let mood = $state<'idle' | 'thinking' | 'happy'>('idle');
</script>

<div class="flex flex-col gap-6">
	<Segmented
		label="Mood"
		options={[
			{ value: 'idle', label: 'Idle' },
			{ value: 'thinking', label: 'Thinking' },
			{ value: 'happy', label: 'Happy' },
		]}
		bind:value={() => mood, (next) => (mood = next as typeof mood)}
	/>
	<div class="flex items-end gap-6">
		<Mascot agent="nomi" label="Nomi" {mood} />
		<Mascot agent="koda" label="Koda" {mood} />
	</div>
	<div class="grid grid-cols-2 gap-x-6 gap-y-4 sm:grid-cols-5">
		{#each CREW as member (member.id)}
			<span class="flex items-center gap-2">
				<Mascot agent={member.id} size={32} {mood} />
				<span class="text-ui text-text-2">{member.name}</span>
			</span>
		{/each}
	</div>
</div>
