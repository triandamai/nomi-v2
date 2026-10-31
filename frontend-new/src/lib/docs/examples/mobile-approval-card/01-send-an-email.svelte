<!-- Send an email: who wants to do what, a preview, and two full-size buttons. -->
<script lang="ts">
	import { Send } from '@lucide/svelte';
	import ApprovalCard from '$lib/ds/mobile/ApprovalCard.svelte';
</script>

<div class="w-89.5 max-w-full">
	<ApprovalCard
		agent="tara"
		title="Tara wants to send an email"
		to="bu.sri.kosmawar@gmail.com"
		subject="Transfer sewa November sudah masuk"
		excerpt="Selamat siang Bu Sri, sewa kamar 3 untuk November (Rp 2.200.000) sudah saya transfer ke BCA. Bukti terlampir. Terima kasih."
		attachment="bukti-transfer-nov.jpg"
		approveLabel="Approve"
		approveIcon={Send}
		rejectLabel="Don't send"
		onapprove={() => {}}
		onreject={() => {}}
		onopen={() => {}}
	/>
</div>
