<!-- With alternatives: what won't happen and why, and what each crew member can do instead. -->
<script lang="ts">
	import Boundary from '$lib/ds/chat/Boundary.svelte';
</script>

<div class="w-full max-w-170">
	<Boundary
		title="I can't cancel subscriptions for you"
		reason="Nomi only records money. It never signs in to Netflix or your bank, so cancelling stays with you. Here's what I can do instead:"
		alternatives={[
			{ agent: 'kala', title: 'Remind me 3 days before the next charge', detail: 'Kala · 2 Nov, 09:00', onclick: () => {} },
			{ agent: 'dana', title: 'Stop tracking it as a subscription', detail: 'Dana · keeps past records', onclick: () => {} },
		]}
	/>
</div>
