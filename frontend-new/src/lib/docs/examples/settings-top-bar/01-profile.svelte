<!-- A settings page, with an action. -->
<script lang="ts">
	import { Link } from '@lucide/svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import SettingsTopBar from '$lib/ds/chrome/SettingsTopBar.svelte';
</script>

<div class="w-full max-w-240 border-b border-border px-6 py-2.75">
	<SettingsTopBar section="Settings" page="Profile">
		{#snippet actions()}<Button label="Copy link" variant="secondary" iconEnd={Link} />{/snippet}
	</SettingsTopBar>
</div>
