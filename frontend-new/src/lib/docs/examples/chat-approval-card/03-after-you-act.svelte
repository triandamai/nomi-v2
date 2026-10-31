<!-- After you act: a one-line outcome that stays in the thread. -->
<script lang="ts">
	import ApprovalCard from '$lib/ds/chat/ApprovalCard.svelte';
</script>

<div class="flex w-full max-w-170 flex-col gap-5">
	<ApprovalCard
		agent="tara"
		title="Tara sent the email to Bu Sri"
		status="approved"
		outcome="Approved by you · sent 14:06"
		action={{ label: 'View in Gmail', onclick: () => {} }}
	/>
	<ApprovalCard
		agent="tara"
		title="Tara's email to Bu Sri"
		status="rejected"
		outcome="You chose not to send · nothing left your Gmail"
		action={{ label: 'Undo', onclick: () => {} }}
	/>
</div>
