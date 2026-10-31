<!-- Next questions: optional, under an answer; picking one asks it. -->
<script lang="ts">
	import FollowUps from '$lib/ds/chat/FollowUps.svelte';
</script>

<div class="w-full max-w-170">
	<FollowUps items={['Compare with the last three months', 'Set a weekly food budget of Rp 500rb', 'Show only GoFood orders after 21:00']} onpick={() => {}} />
</div>
