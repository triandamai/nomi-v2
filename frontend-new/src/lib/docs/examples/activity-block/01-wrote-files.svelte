<!-- Wrote files: the sum in one line, the real files under it. -->
<script lang="ts">
	import ActivityBlock from '$lib/ds/conversation/ActivityBlock.svelte';
</script>

<div class="w-full max-w-120">
	<ActivityBlock
		title="Wrote 3 files"
		total="+424"
		files={[
			{ path: 'src/routes/+page.svelte', change: '+214' },
			{ path: 'src/lib/prayer-times.ts', change: '+162' },
			{ path: 'src/lib/storage.ts', change: '+48' },
		]}
	/>
</div>
