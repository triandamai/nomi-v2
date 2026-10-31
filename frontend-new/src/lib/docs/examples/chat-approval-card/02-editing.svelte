<!-- Editing: your words go out exactly as written. -->
<script lang="ts">
	import { Send } from '@lucide/svelte';
	import ApprovalCard from '$lib/ds/chat/ApprovalCard.svelte';
</script>

<div class="w-full max-w-170">
	<ApprovalCard
		agent="tara"
		title="Editing Tara's email"
		status="editing"
		fields={[
			{ key: 'to', value: 'bu.sri.kosmawar@gmail.com' },
			{ key: 'subject', value: 'Transfer sewa November sudah masuk' },
		]}
		body="Selamat siang Bu Sri, sewa kamar 3 untuk November (Rp 2.200.000) sudah saya transfer ke BCA. Bukti terlampir. Mohon maaf telat dua hari ya, Bu. Terima kasih."
		approveLabel="Save & send"
		approveIcon={Send}
		onsave={() => {}}
		oncancel={() => {}}
	/>
</div>
