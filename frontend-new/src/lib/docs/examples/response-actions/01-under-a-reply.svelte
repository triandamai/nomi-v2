<!-- Under a reply: like and dislike are pressed toggles; model, time and tokens on the right. -->
<script lang="ts">
	import ReplyText from '$lib/ds/chat/ReplyText.svelte';
	import ResponseActions from '$lib/ds/chat/ResponseActions.svelte';

	const reply = "Food is October's biggest jump: Rp 2,1 jt against Rp 1,4 jt in September. Most of it is GoFood after 21:00 on weekdays…";
	let feedback = $state<'liked' | 'disliked' | null>(null);
</script>

<div class="flex w-full max-w-170 flex-col gap-2">
	<ReplyText text={reply} />
	<ResponseActions
		text={reply}
		meta="deepseek-flash · 1.4s · 812 tokens"
		{feedback}
		onlike={() => (feedback = feedback === 'liked' ? null : 'liked')}
		ondislike={() => (feedback = feedback === 'disliked' ? null : 'disliked')}
		onshare={() => {}}
		onregenerate={() => {}}
		ondelete={() => {}}
	/>
</div>
