<!-- On a phone: a failed file offers its sheet (⋯) in place of Retry and Remove. -->
<script lang="ts">
	import ComposerFile from '$lib/ds/thread/ComposerFile.svelte';
</script>

<ComposerFile phone name="bca-oct-2026.pdf" status="failed" detail="Upload failed · connection lost" onretry={() => {}} onremove={() => {}} onmore={() => {}} />
