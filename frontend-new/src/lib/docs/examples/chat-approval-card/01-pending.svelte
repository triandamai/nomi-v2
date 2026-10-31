<!-- Pending: exactly what will go out, waiting for you. -->
<script lang="ts">
	import { Send } from '@lucide/svelte';
	import ApprovalCard from '$lib/ds/chat/ApprovalCard.svelte';
</script>

<div class="w-full max-w-170">
	<ApprovalCard
		agent="tara"
		title="Tara wants to send an email"
		fields={[
			{ key: 'to', value: 'bu.sri.kosmawar@gmail.com' },
			{ key: 'from', value: 'you, via Gmail' },
			{ key: 'subject', value: 'Transfer sewa November sudah masuk' },
		]}
		body="Selamat siang Bu Sri, sewa kamar 3 untuk November (Rp 2.200.000) sudah saya transfer ke BCA. Bukti terlampir. Terima kasih."
		attachment="bukti-transfer-nov.jpg · 212 KB"
		rememberLabel="Don't ask again for Bu Sri"
		approveLabel="Approve & send"
		approveIcon={Send}
		rejectLabel="Don't send"
		onapprove={() => {}}
		onreject={() => {}}
		onedit={() => {}}
	/>
</div>
