<script lang="ts">
	import FirstRun from '$lib/ds/build/FirstRun.svelte';
	import GallerySection from '../GallerySection.svelte';
	import GalleryBoard from '../GalleryBoard.svelte';
</script>

<GallerySection title="First run" description="One screen after the code: a name and one line on what Nomi does. No crew tour; the composer comes next.">
	<div class="grid items-start gap-6 lg:grid-cols-2">
		<GalleryBoard title="First run" note="≤ 1 input" inline>
			<FirstRun name="Mario Prasetyo" oncontinue={() => {}} />
		</GalleryBoard>
	</div>
</GallerySection>
