<!-- Change your email: the field, what happens next, and Send code at the foot. -->
<script lang="ts">
	import PhoneFormScreen from '$lib/ds/settings/PhoneFormScreen.svelte';
	import PhoneButton from '$lib/ds/mobile/PhoneButton.svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';

	let email = $state('');
</script>

<div class="flex h-120 w-97.5 max-w-full flex-col overflow-y-auto">
	<PhoneFormScreen backLabel="General" onback={() => {}} title="Change your email" sub="We’ll send a code to the new address. Your email stays as it is until you enter it." onsubmit={() => {}}>
		<TextField label="New email" type="email" bind:value={email} />
		{#snippet action()}<PhoneButton label="Send code" disabled={!email.trim()} />{/snippet}
	</PhoneFormScreen>
</div>
