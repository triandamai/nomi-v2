<!-- The preview didn't load: what happened, its code, Try again and Download. -->
<script lang="ts">
	import FileViewer from '$lib/ds/thread/FileViewer.svelte';
</script>

<div class="h-160">
	<FileViewer
		name="bca-sep-2026.pdf"
		meta="PDF · 12 pages · your attachment"
		status="error"
		error={{ body: 'Your file is fine and still in this chat. Try again, or download it and open page 1 yourself.', code: 'preview_timeout · 30 s' }}
		downloadHref="#"
		onretry={() => {}}
		onclose={() => {}}
	/>
</div>
