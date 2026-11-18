<!-- An email, in full, before it goes. -->
<script lang="ts">
	import PhoneApprovalSheet from '$lib/ds/thread/PhoneApprovalSheet.svelte';
	import Button from '$lib/ds/actions/Button.svelte';

	let open = $state(false);
</script>

<Button variant="secondary" label="Open the approval" onclick={() => (open = true)} />
<PhoneApprovalSheet
	bind:open
	title="Send this email?"
	subtitle="Tara · via your Gmail"
	fields={[
		{ key: 'To', value: 'bu.sri.kosmawar@gmail.com' },
		{ key: 'Subject', value: 'Transfer sewa November sudah masuk' },
	]}
	body="Selamat siang Bu Sri, sewa kamar 3 untuk November (Rp 2.200.000) sudah saya transfer ke BCA."
	attachment="bukti-transfer-nov.jpg · 212 KB"
	approveLabel="Approve & send"
	onapprove={() => (open = false)}
/>
