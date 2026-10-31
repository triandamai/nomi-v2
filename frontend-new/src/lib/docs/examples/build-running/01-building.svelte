<!-- Building: the real step and its time, a stop, and which of the plan's steps are done. -->
<script lang="ts">
	import BuildRunning from '$lib/ds/build/BuildRunning.svelte';
</script>

<div class="w-full max-w-140">
	<BuildRunning started="08:13" step="writing src/routes/menu/+page.svelte" elapsed="01:12" current={2} total={6} label="menu page" eta="~3 min left" onstop={() => {}} />
</div>
