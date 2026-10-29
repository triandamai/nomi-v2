<script lang="ts">
	import ChatMenuFixture from '$lib/ds/chat/ChatMenuFixture.svelte';
	import RenameField from '$lib/ds/chat/RenameField.svelte';
	import MoveDialog from '$lib/ds/chat/MoveDialog.svelte';
	import JumpToLatest from '$lib/ds/chat/JumpToLatest.svelte';
	import NewDivider from '$lib/ds/chat/NewDivider.svelte';
	import LoadingOlder from '$lib/ds/chat/LoadingOlder.svelte';
	import DateDivider from '$lib/ds/chat/DateDivider.svelte';
	import SkeletonRow from '$lib/ds/lists/SkeletonRow.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import SectionLabel from '$lib/ds/status/SectionLabel.svelte';
	import GallerySection from '../GallerySection.svelte';
	import GalleryBoard from '../GalleryBoard.svelte';

	const noop = () => {};
	let moving = $state(false);
</script>

<GallerySection
	title="Chat management & long threads"
	description="The chat itself can be renamed, pinned, moved into a project, exported or deleted. Long threads keep your place and tell you when something new arrives."
>
	<div class="grid items-start gap-x-8 gap-y-6 lg:grid-cols-2">
		<GalleryBoard title="Chat menu" note="top bar ··· · rename · move">
			<div class="flex flex-col items-start gap-4">
				<SectionLabel label="Menu" />
				<ChatMenuFixture onpick={(action) => action === 'move' && (moving = true)} />
				<SectionLabel label="Rename in top bar" />
				<RenameField value="Food spending in October" onsave={noop} oncancel={noop} />
				<SectionLabel label="Move to a project" />
				<Button label="Move chat…" variant="secondary" onclick={() => (moving = true)} />
			</div>
		</GalleryBoard>

		<GalleryBoard title="Long threads" note="scroll · new · older">
			<div class="flex flex-col gap-4">
				<SectionLabel label="Jump to latest" />
				<div><JumpToLatest count={2} onclick={noop} /></div>
				<SectionLabel label="New since you left" />
				<NewDivider count={2} />
				<SectionLabel label="Loading older" />
				<LoadingOlder />
				<SkeletonRow label="loading 4 projects…" />
				<SectionLabel label="Date divider" />
				<DateDivider label="Yesterday" />
			</div>
		</GalleryBoard>
	</div>
</GallerySection>

<MoveDialog
	bind:open={moving}
	projects={[
		{ id: 'budget-keluarga', name: 'budget-keluarga', workspace: 'Keluarga workspace', shared: 'Keluarga' },
		{ id: 'salah-daily-tracker', name: 'salah-daily-tracker', workspace: 'Personal' },
		{ id: 'kos-mawar-tenants', name: 'kos-mawar-tenants', workspace: 'Personal' },
	]}
	onmove={noop}
/>
