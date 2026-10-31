<!-- Passed on: who handed it to whom, and why; you can send it back. -->
<script lang="ts">
	import Handoff from '$lib/ds/chat/Handoff.svelte';
</script>

<div class="w-full max-w-170">
	<Handoff from={{ agent: 'nomi', name: 'Nomi' }} to={{ agent: 'dana', name: 'Dana' }} reason="she has your transaction records" onback={() => {}} />
</div>
