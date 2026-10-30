<script lang="ts">
	import { Trash2 } from '@lucide/svelte';
	import ResponseActions from '$lib/ds/chat/ResponseActions.svelte';
	import FeedbackSent from '$lib/ds/chat/FeedbackSent.svelte';
	import DeletedReply from '$lib/ds/chat/DeletedReply.svelte';
	import DislikeFeedback from '$lib/ds/chat/DislikeFeedback.svelte';
	import SharePanel from '$lib/ds/chat/SharePanel.svelte';
	import ShareDialog from '$lib/ds/chat/ShareDialog.svelte';
	import DeleteReplyDialog from '$lib/ds/chat/DeleteReplyDialog.svelte';
	import Toast from '$lib/ds/overlays/Toast.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import SectionLabel from '$lib/ds/status/SectionLabel.svelte';
	import GallerySection from '../GallerySection.svelte';
	import GalleryBoard from '../GalleryBoard.svelte';

	const reply = "Food is October's biggest jump: Rp 2,1 jt against Rp 1,4 jt in September. Most of it is GoFood after 21:00 on weekdays…";
	const meta = 'deepseek-flash · 1.4s · 812 tokens';

	let feedback = $state<'liked' | 'disliked' | null>(null);
	let masked = $state(true);
	let shareOpen = $state(false);
	let deleteOpen = $state(false);

	const preview = $derived({
		agent: 'nomi' as const,
		name: 'Nomi',
		time: '14:02',
		excerpt: masked ? reply.replace(/Rp [\d,]+ jt/g, 'Rp •••') : reply,
	});
	const share = $derived({ preview, chatMessages: 6, url: 'https://nomi.app/s/k7Qe2-r4', expiry: 'in 7 days' });
</script>

<GallerySection
	title="Response feedback"
	description="Every reply ends with the same quiet bar. Like is one tap; dislike asks why; share masks your numbers by default; delete confirms and can be undone."
>
	<div class="grid items-start gap-x-8 gap-y-6 lg:grid-cols-2">
		<GalleryBoard title="Response actions" note="default · liked · disliked">
			<div class="flex flex-col gap-4">
				<SectionLabel label="Default" />
				<ResponseActions
					text={reply}
					{meta}
					{feedback}
					onlike={() => (feedback = feedback === 'liked' ? null : 'liked')}
					ondislike={() => (feedback = feedback === 'disliked' ? null : 'disliked')}
					onshare={() => (shareOpen = true)}
					onregenerate={() => {}}
					ondelete={() => (deleteOpen = true)}
				/>
				<SectionLabel label="Liked" />
				<ResponseActions text={reply} {meta} feedback="liked" onlike={() => {}} ondislike={() => {}} onshare={() => {}} onregenerate={() => {}} ondelete={() => {}} />
				<SectionLabel label="Disliked" />
				<ResponseActions text={reply} {meta} feedback="disliked" onlike={() => {}} ondislike={() => {}} onshare={() => {}} onregenerate={() => {}} ondelete={() => {}} />
				<SectionLabel label="After feedback / delete" />
				<FeedbackSent retry={{ label: 'Try again with Claude Sonnet', onclick: () => {} }} />
				<DeletedReply name="Nomi" onundo={() => {}} onregenerate={() => {}} />
			</div>
		</GalleryBoard>

		<GalleryBoard title="Dislike" note="reason, then optional detail">
			<DislikeFeedback onsend={() => {}} oncancel={() => {}} />
		</GalleryBoard>

		<GalleryBoard title="Share" note="masked by default">
			<div class="flex flex-col items-start gap-4">
				<div class="flex w-115 max-w-full flex-col overflow-hidden rounded-md border border-border-strong bg-surface shadow-float">
					<SharePanel {...share} onchange={(settings) => (masked = settings.mask)} onclose={() => {}} />
				</div>
				<Button label="Open as a dialog" variant="secondary" onclick={() => (shareOpen = true)} />
			</div>
		</GalleryBoard>

		<GalleryBoard title="Delete & toasts" note="confirm · undo">
			<div class="flex flex-col items-start gap-4">
				<SectionLabel label="Confirm" />
				<Button label="Delete this reply…" variant="secondary" onclick={() => (deleteOpen = true)} />
				<SectionLabel label="Toasts" />
				<Toast kind="success" title="Link copied" detail="nomi.app/s/k7Qe2-r4 · expires in 7 days" action={{ label: 'Revoke', onclick: () => {} }} ondismiss={() => {}} />
				<Toast kind="info" icon={Trash2} title="Reply deleted" detail="Undo within 10 seconds" action={{ label: 'Undo', onclick: () => {} }} ondismiss={() => {}} />
				<Toast kind="error" title="Couldn't create the link" detail="You're offline. Try again when you reconnect." action={{ label: 'Retry', onclick: () => {} }} ondismiss={() => {}} />
			</div>
		</GalleryBoard>
	</div>
</GallerySection>

<ShareDialog bind:open={shareOpen} {...share} onchange={(settings) => (masked = settings.mask)} />
<DeleteReplyDialog bind:open={deleteOpen} name="Nomi" ondelete={() => {}} />
