<script lang="ts">
	import { Cpu, Sparkles } from '@lucide/svelte';
	import MessageUser from '$lib/ds/conversation/MessageUser.svelte';
	import VersionSwitcher from '$lib/ds/chat/VersionSwitcher.svelte';
	import MessageEditor from '$lib/ds/chat/MessageEditor.svelte';
	import RegenerateMenu from '$lib/ds/chat/RegenerateMenu.svelte';
	import AttachMenu from '$lib/ds/chat/AttachMenu.svelte';
	import CopiedNote from '$lib/ds/chat/CopiedNote.svelte';
	import StopHint from '$lib/ds/chat/StopHint.svelte';
	import AttachmentChip from '$lib/ds/chat/AttachmentChip.svelte';
	import AttachmentImage from '$lib/ds/chat/AttachmentImage.svelte';
	import VoiceRecorder from '$lib/ds/chat/VoiceRecorder.svelte';
	import VoiceTranscribed from '$lib/ds/chat/VoiceTranscribed.svelte';
	import ComposerLimit from '$lib/ds/chat/ComposerLimit.svelte';
	import SectionLabel from '$lib/ds/status/SectionLabel.svelte';
	import GallerySection from '../GallerySection.svelte';
	import GalleryBoard from '../GalleryBoard.svelte';

	const noop = () => {};
	// The pen's waveform, in pixels of a 28px-high track: thirty recorded, the last ten settling.
	const heights = [6, 10, 16, 22, 14, 8, 18, 26, 20, 12, 6, 9, 15, 24, 28, 18, 10, 7, 12, 20, 16, 9, 5, 8, 14, 22, 18, 11, 6, 4, 6, 10, 14, 8, 5, 4, 4, 4, 4, 4];
	const levels = heights.map((height) => (height - 4) / 24);
	// The pen's own pictures: a receipt in the composer, a bank statement sent with the message.
	const receipt = 'https://images.unsplash.com/photo-1721320147450-c57904a10999?crop=entropy&cs=tinysrgb&fit=max&fm=jpg&q=80&w=200';
	const statement = 'https://images.unsplash.com/photo-1587602526256-e0ca9705305e?crop=entropy&cs=tinysrgb&fit=max&fm=jpg&q=80&w=400';
</script>

<GallerySection
	title="Message controls & composer"
	description="Every message can be stopped, edited, regenerated or copied. Edits branch instead of overwriting. Attachments and voice show progress and limits before you send."
>
	<div class="grid items-start gap-x-8 gap-y-6 lg:grid-cols-2">
		<GalleryBoard title="Message actions" note="hover · edit · branch">
			<div class="flex flex-col gap-5">
				<VersionSwitcher current={2} total={2} onprev={noop} onnext={noop} />
				<SectionLabel label="Hover" />
				<MessageUser text="How much did I spend on food in October?" time="14:02" onedit={noop} data-hover />
				<SectionLabel label="Editing" />
				<MessageEditor value="How much did I spend on food in October, without office lunches?" version="1 / 2" onsave={noop} oncancel={noop} />
				<SectionLabel label="Edited" />
				<MessageUser text="How much did I spend on food in October, without office lunches?" time="14:05" edited version={{ current: 2, total: 2, onprev: noop, onnext: noop }} />
			</div>
		</GalleryBoard>

		<GalleryBoard title="Regenerate & copy" note="from the action bar">
			<div class="flex flex-col items-start gap-5">
				<SectionLabel label="Regenerate menu" />
				<RegenerateMenu
					inline
					models={[
						{ name: 'claude-sonnet', source: 'your key', icon: Cpu },
						{ name: 'deepseek-flash', source: 'Nomi credits', icon: Sparkles },
					]}
					onpick={noop}
				/>
				<SectionLabel label="Copied" />
				<CopiedNote label="Copied as Markdown" />
				<SectionLabel label="Stop" />
				<StopHint onstop={noop} />
			</div>
		</GalleryBoard>

		<GalleryBoard title="Attachments" note="menu · uploading · sent">
			<div class="flex flex-col gap-5">
				<SectionLabel label="Attach menu" />
				<AttachMenu inline driveBy="Tara" max="20 MB" onpick={noop} />
				<SectionLabel label="Chips" />
				<div class="flex flex-wrap items-center gap-2">
					<AttachmentChip name="bca-oct-2026.pdf" progress={0.62} detail="62% · 3,1 of 5 MB" onremove={noop} />
					<AttachmentChip name="bca-sep-2026.pdf" detail="PDF · 4,2 MB" onremove={noop} />
					<AttachmentImage src={receipt} alt="Receipt photo" onremove={noop} />
				</div>
				<SectionLabel label="Sent with files" />
				<MessageUser
					text="What did I spend on rent and food these two months?"
					files={[
						{ name: 'bca-sep-2026.pdf', detail: 'PDF · 12 pages' },
						{ name: 'bca-oct-2026.pdf', detail: 'PDF · 11 pages' },
					]}
					images={[{ src: statement, alt: 'Bank statement photo' }]}
				/>
			</div>
		</GalleryBoard>

		<GalleryBoard title="Voice & limits" note="record · transcribe · too long">
			<div class="flex flex-col gap-5">
				<SectionLabel label="Recording" />
				<VoiceRecorder elapsed="00:14" {levels} fading={10} language="Bahasa Indonesia" oncancel={noop} ondone={noop} />
				<SectionLabel label="Transcribed" />
				<VoiceTranscribed duration="0:14" />
				<SectionLabel label="Too long" />
				<ComposerLimit count="12.480" max="10.000" onfile={noop} />
			</div>
		</GalleryBoard>
	</div>
</GallerySection>
