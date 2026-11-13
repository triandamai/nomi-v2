<!-- Default: a picture of the app, who made it, and Remix. -->
<script lang="ts">
	import RemixCard from '$lib/ds/projects/RemixCard.svelte';
</script>

<RemixCard name="Kos Melati rent tracker" maker="Bu Sari · Yogyakarta · 214 remixes" crew={['dana', 'kala']} tone="dana" onremix={() => {}}>
	{#snippet preview()}
		<div class="flex h-25 flex-1 flex-col gap-1.75 rounded-t-inner bg-surface px-3.5 py-2.5">
			<p class="flex justify-between text-micro text-text">Room 1 <span class="font-mono text-nano text-accent">paid</span></p>
			<p class="flex justify-between text-micro text-text">Room 2 <span class="font-mono text-nano text-text-3">due</span></p>
		</div>
	{/snippet}
</RemixCard>
