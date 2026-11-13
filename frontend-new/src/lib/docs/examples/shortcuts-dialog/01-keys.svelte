<!-- The keys: only the ones the app answers to. -->
<script lang="ts">
	import Button from '$lib/ds/actions/Button.svelte';
	import ShortcutsDialog from '$lib/ds/chrome/ShortcutsDialog.svelte';

	let open = $state(false);
</script>

<Button label="Keyboard shortcuts" variant="secondary" onclick={() => (open = true)} />
<ShortcutsDialog
	bind:open
	shortcuts={[
		{ label: 'New chat', keys: '⌘K' },
		{ label: 'Settings', keys: '⌘ ,' },
		{ label: 'Show keyboard shortcuts', keys: '?' },
		{ label: 'Close a menu or dialog', keys: 'esc' },
	]}
/>
