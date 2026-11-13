<!-- Changed, and nothing changed. -->
<script lang="ts">
	import SaveBar from '$lib/ds/profile/SaveBar.svelte';
</script>

<div class="flex w-full max-w-80 flex-col gap-3">
	<SaveBar pending="1 unsaved change" dirty discard="Discard" save="Save" />
	<SaveBar pending="" dirty={false} discard="Discard" save="Save" />
</div>
