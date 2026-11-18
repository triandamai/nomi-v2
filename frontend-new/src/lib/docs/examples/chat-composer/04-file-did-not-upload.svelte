<!-- A file didn't upload: its chip says why, and Send waits with the reason beside it. -->
<script lang="ts">
	import ChatComposer from '$lib/ds/thread/ChatComposer.svelte';
	import ComposerFile from '$lib/ds/thread/ComposerFile.svelte';
</script>

<div class="max-w-180">
	<ChatComposer placeholder="Ask anything…" value="What did I spend on rent and food these two months?" blocked="1 file didn’t upload. Retry or remove it to send." onsend={() => {}} onattach={() => {}}>
		{#snippet files()}
			<div class="flex flex-wrap items-center gap-2">
				<ComposerFile name="bca-sep-2026.pdf" status="ready" detail="PDF · 4,2 MB" onremove={() => {}} />
				<ComposerFile name="bca-oct-2026.pdf" status="failed" detail="Upload failed · connection lost" onretry={() => {}} onremove={() => {}} />
			</div>
		{/snippet}
	</ChatComposer>
</div>
