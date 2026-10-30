<script lang="ts">
	import Mascot from '$lib/ds/agents/Mascot.svelte';
	import { CREW } from '$lib/ds/agents/crew';
	import GallerySection from './GallerySection.svelte';
</script>

<GallerySection
	title="Agents"
	description="One mascot per crew member, flat colour, used only where that member is speaking or working. 96 for headers and empty states, 32 in lists, 20 inline."
>
	<div class="grid grid-cols-5 gap-6">
		{#each CREW as member (member.id)}
			<div class="flex flex-col items-start gap-2">
				<Mascot agent={member.id} />
				<span class="flex items-end gap-2">
					<Mascot agent={member.id} size={32} />
					<Mascot agent={member.id} size={20} />
				</span>
				<span class="text-ui font-medium text-text">{member.name}</span>
				<span class="font-mono text-label text-text-3">crew-{member.id}</span>
			</div>
		{/each}
	</div>
</GallerySection>
