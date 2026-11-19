<!-- A live app's share sheet: press Share to open it. -->
<script lang="ts">
	import PhoneBuildShareSheet from '$lib/ds/projects/PhoneBuildShareSheet.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	let open = $state(false);
</script>

<Button label="Share" onclick={() => (open = true)} />
<PhoneBuildShareSheet bind:open project="warung-kopi-menu" url="https://warung-kopi-menu-3f9a12.trian.space" oninvite={() => {}} />
