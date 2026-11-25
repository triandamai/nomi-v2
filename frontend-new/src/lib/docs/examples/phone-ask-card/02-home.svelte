<!-- On Home: room to write, and a + that attaches files. -->
<script lang="ts">
	import PhoneAskCard from '$lib/ds/thread/PhoneAskCard.svelte';
	import ComposerFile from '$lib/ds/thread/ComposerFile.svelte';

	let value = $state('');
</script>

<div class="w-97.5 max-w-full">
	<PhoneAskCard bind:value placeholder="Ask Nomi, or hand something to the crew…" to={{ agent: 'nomi', name: 'Nomi' }} model="Gemini 3.6 Flash" roomy attached onattach={() => {}} onsend={() => (value = '')}>
		{#snippet files()}
			<div class="flex items-center gap-1.5 pt-1.5 pr-1.5">
				<ComposerFile name="bca-oct-2026.pdf" status="ready" detail="PDF · 412 KB" phone onremove={() => {}} />
			</div>
		{/snippet}
	</PhoneAskCard>
</div>
