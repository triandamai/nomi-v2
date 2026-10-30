<script lang="ts">
	import ResizeHandle from '$lib/ds/build/ResizeHandle.svelte';
	import PublishButton from '$lib/ds/build/PublishButton.svelte';
	import WorkspaceSheetFixture from '$lib/ds/build/WorkspaceSheetFixture.svelte';
	import GallerySection from '../GallerySection.svelte';
	import GalleryBoard from '../GalleryBoard.svelte';

	const noop = () => {};
	let width = $state(520);
</script>

<GallerySection
	title="Workspace"
	description="Chat and preview share one screen. The split is draggable, and every project tool opens as a sheet over the preview, so the app you are looking at never closes."
>
	<div class="grid items-start gap-x-8 gap-y-6 lg:grid-cols-2">
		<div class="flex flex-col gap-6">
			<GalleryBoard title="Resize handle" note="default · hover · dragging" inline>
				<div class="flex h-40 gap-25">
					<ResizeHandle {width} available={1600} onresize={(next) => (width = next)} />
					<ResizeHandle {width} available={1600} onresize={noop} data-hover />
					<ResizeHandle {width} available={1600} onresize={noop} data-dragging />
				</div>
				<p class="text-body leading-normal text-text-3">
					Drag to resize. Double-click resets to 520 / rest. Chat stays between 360 and 720 px; the preview never goes below 480.
				</p>
			</GalleryBoard>
			<GalleryBoard title="Publish" note="top bar · four states">
				<div class="flex flex-col gap-3">
					<p class="flex flex-wrap items-center gap-3"><PublishButton state="idle" /><span class="font-mono text-caption text-text-3">Up to date · nothing waiting to go live</span></p>
					<p class="flex flex-wrap items-center gap-3"><PublishButton state="ready" count={2} onpublish={noop} /><span class="font-mono text-caption text-text-3">Ready · the count is what will go live</span></p>
					<p class="flex flex-wrap items-center gap-3"><PublishButton state="publishing" elapsed="0:12" /><span class="font-mono text-caption text-text-3">Publishing · the live site keeps running</span></p>
					<p class="flex flex-wrap items-center gap-3"><PublishButton state="failed" onretry={noop} /><span class="font-mono text-caption text-text-3">Failed · the last live deploy stays up</span></p>
				</div>
				<p class="text-body leading-normal text-text-3">
					Changes stay in the preview until you press Publish. The count is what will go live. After a publish the button settles to Up to date; a
					failed publish keeps the last live deploy running.
				</p>
			</GalleryBoard>
		</div>
		<GalleryBoard title="Sheet over preview" note="opens from the project nav, closes back to the same preview" inline>
			<WorkspaceSheetFixture />
		</GalleryBoard>
	</div>
</GallerySection>
