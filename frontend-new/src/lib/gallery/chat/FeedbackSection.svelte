<script lang="ts">
	import { Trash2 } from '@lucide/svelte';
	import ResponseActions from '$lib/ds/chat/ResponseActions.svelte';
	import FeedbackSent from '$lib/ds/chat/FeedbackSent.svelte';
	import DeletedReply from '$lib/ds/chat/DeletedReply.svelte';
	import DislikeFeedback from '$lib/ds/chat/DislikeFeedback.svelte';
	import ShareDialog from '$lib/ds/chat/ShareDialog.svelte';
	import DeleteReplyDialog from '$lib/ds/chat/DeleteReplyDialog.svelte';
	import Toast from '$lib/ds/overlays/Toast.svelte';
	import SectionLabel from '$lib/ds/status/SectionLabel.svelte';
	import GallerySection from '../GallerySection.svelte';
	import GalleryBoard from '../GalleryBoard.svelte';

	const reply = "Food is October's biggest jump: Rp 2,1 jt against Rp 1,4 jt in September. Most of it is GoFood after 21:00 on weekdays…";
	const meta = 'deepseek-flash · 1.4s · 812 tokens';

	let feedback = $state<'liked' | 'disliked' | null>(null);
	let masked = $state(true);

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
			<div class="flex flex-col gap-5">
				<SectionLabel label="Default" />
				<ResponseActions
					text={reply}
					{meta}
					{feedback}
					onlike={() => (feedback = feedback === 'liked' ? null : 'liked')}
					ondislike={() => (feedback = feedback === 'disliked' ? null : 'disliked')}
					onshare={() => {}}
					onregenerate={() => {}}
					ondelete={() => {}}
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
			<DislikeFeedback
				initialReasons={['inaccurate', 'wrong_crew']}
				initialNote="It counted the Rp 300rb of office lunches my work pays back. Those aren't my spending."
				onsend={() => {}}
				oncancel={() => {}}
			/>
		</GalleryBoard>

		<GalleryBoard title="Share" note="masked by default">
			<ShareDialog inline {...share} onchange={(settings) => (masked = settings.mask)} />
		</GalleryBoard>

		<GalleryBoard title="Delete & toasts" note="confirm · undo">
			<div class="flex flex-col items-start gap-5">
				<SectionLabel label="Confirm" />
				<DeleteReplyDialog inline name="Nomi" ondelete={() => {}} />
				<SectionLabel label="Toasts" />
				<Toast kind="success" title="Link copied" detail="nomi.app/s/k7Qe2-r4 · expires in 7 days" action={{ label: 'Revoke', onclick: () => {} }} ondismiss={() => {}} />
				<Toast kind="info" icon={Trash2} title="Reply deleted" detail="Undo within 10 seconds" action={{ label: 'Undo', onclick: () => {} }} ondismiss={() => {}} />
				<Toast kind="error" title="Couldn't create the link" detail="You're offline. Try again when you reconnect." action={{ label: 'Retry', onclick: () => {} }} ondismiss={() => {}} />
			</div>
		</GalleryBoard>
	</div>
</GallerySection>

