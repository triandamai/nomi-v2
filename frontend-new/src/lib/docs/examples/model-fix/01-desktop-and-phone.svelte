<!-- The fix line: a desktop's one row, then a phone's line with its two pills. -->
<script lang="ts">
	import ModelFix from '$lib/ds/thread/ModelFix.svelte';
</script>

<div class="flex max-w-180 flex-col gap-6">
	<ModelFix onswitch={() => {}} ontextonly={() => {}} />
	<div class="max-w-97.5"><ModelFix phone onswitch={() => {}} ontextonly={() => {}} /></div>
</div>
