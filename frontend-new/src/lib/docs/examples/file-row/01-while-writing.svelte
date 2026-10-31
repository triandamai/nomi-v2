<!-- While a build writes files: written, being written, still to come. -->
<script lang="ts">
	import FileRow from '$lib/ds/conversation/FileRow.svelte';
</script>

<div class="flex w-full max-w-100 flex-col">
	<FileRow path="src/routes/+page.svelte" change="+214" state="done" />
	<FileRow path="src/routes/menu/+page.svelte" state="writing" />
	<FileRow path="src/lib/storage.ts" state="queued" />
</div>
