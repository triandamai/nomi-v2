<!-- The crew: one colour and one silhouette each. Beside a name, the mascot is decorative. -->
<script lang="ts">
	import Mascot from '$lib/ds/agents/Mascot.svelte';
	import { CREW } from '$lib/ds/agents/crew';
</script>

<div class="grid grid-cols-2 gap-x-6 gap-y-4 sm:grid-cols-5">
	{#each CREW as member (member.id)}
		<span class="flex items-center gap-2">
			<Mascot agent={member.id} size={32} />
			<span class="text-ui text-text-2">{member.name}</span>
		</span>
	{/each}
</div>
