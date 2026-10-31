<!-- Working: who is on this chat right now, with one way to stop them all. -->
<script lang="ts">
	import CrewWorking from '$lib/ds/chat/CrewWorking.svelte';
</script>

<div class="w-full max-w-170">
	<CrewWorking crew={['nomi', 'dana', 'tara']} onstop={() => {}} />
</div>
