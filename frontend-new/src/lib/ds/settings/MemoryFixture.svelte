<script lang="ts">
	import MemoryTable from './MemoryTable.svelte';
	import MemoryTopicRow from './MemoryTopicRow.svelte';
	import MemoryEntry from './MemoryEntry.svelte';
	import type { CrewId } from '../agents/crew';

	// For the tests: a topic, open, with one memory the crew asks about; 820 wide as drawn.
	let {
		onsave,
		onforget,
		onconfirm,
		learnedBy,
	}: { onsave?: (content: string) => void; onforget?: () => void; onconfirm?: () => void; learnedBy?: { agent?: CrewId; name: string } } = $props();
</script>

<div class="w-205">
	<MemoryTable columns={{ topic: 'Topic', knows: 'What Nomi knows', ...(learnedBy && { learnedBy: 'Learned by' }), strength: 'Strength', updated: 'Updated' }} note="Every memory starts at Steady.">
		<MemoryTopicRow name="People" count="1 memory" summary="Siti is my sister in Bandung" {learnedBy} strength={{ level: 1, name: 'faint' }} updated="2 Sep" fading open>
			<MemoryEntry
				content="Siti is my sister in Bandung"
				strength={{ level: 1, name: 'faint' }}
				meta="not used yet"
				check={{ question: 'Is this still right?', confirm: 'Still right' }}
				labels={{ edit: 'Correct it', forget: 'Forget it', save: 'Save', cancel: 'Cancel', field: 'What Nomi should remember' }}
				{onsave}
				{onforget}
				{onconfirm}
			/>
		</MemoryTopicRow>
	</MemoryTable>
</div>
