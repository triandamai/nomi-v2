<!-- What became of each file: ready, uploading again after a failure, failed with why, refused, and a picture the model can't see. -->
<script lang="ts">
	import ComposerFile from '$lib/ds/thread/ComposerFile.svelte';

	const receipt = 'data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="28" height="28"><rect width="28" height="28" fill="%23d9d4c7"/></svg>';
</script>

<div class="flex flex-wrap items-center gap-2">
	<ComposerFile name="bca-sep-2026.pdf" status="ready" detail="PDF · 4,2 MB" onremove={() => {}} />
	<ComposerFile name="bca-oct-2026.pdf" status="uploading" progress={0} detail="Uploading again · 0% · 0 of 5 MB" onremove={() => {}} />
	<ComposerFile name="bca-oct-2026.pdf" status="failed" detail="Upload failed · connection lost" onretry={() => {}} onremove={() => {}} />
	<ComposerFile name="kasir-setup.dmg" status="rejected" detail=".dmg files can’t be read" onremove={() => {}} />
	<ComposerFile name="struk-belanja-nov.jpg" status="warning" detail="deepseek-v3 can’t see images" thumb={receipt} onremove={() => {}} />
</div>
