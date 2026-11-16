<!-- Share: this reply, masked. -->
<script lang="ts">
	import PhoneShareSheet from '$lib/ds/thread/PhoneShareSheet.svelte';
	import Button from '$lib/ds/actions/Button.svelte';

	let open = $state(false);
	let settings = $state({ scope: 'reply' as 'reply' | 'chat', mask: true, anyone: true });
</script>

<Button variant="secondary" label="Share the reply" onclick={() => (open = true)} />
<PhoneShareSheet
	bind:open
	excerpt="Rp ••• on food in October, up 20% from September. The jump is GoFood…"
	chatMessages={6}
	url="https://nomi.app/shared/abc"
	expiry="Expires in 7 days"
	{settings}
	onchange={(next) => (settings = next)}
	oncopy={() => {}}
/>
