<!-- What was left out, in mono; a shared link's files kept private, in sans. -->
<script lang="ts">
	import { ImageOff, Lock } from '@lucide/svelte';
	import FileNote from '$lib/ds/thread/FileNote.svelte';
</script>

<div class="flex flex-col gap-3">
	<FileNote icon={ImageOff} mono text="Sent without struk-belanja-nov.jpg · deepseek-v3 can’t see images" />
	<FileNote mono text="bca-oct-2026.pdf removed · 1 file ready" />
	<FileNote icon={Lock} text="M. kept the files in this reply private. You see their names only." />
</div>
