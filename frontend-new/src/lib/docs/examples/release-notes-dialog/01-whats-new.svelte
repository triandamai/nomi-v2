<!-- What's new: this version's notes first, then the earlier ones of the same environment. -->
<script lang="ts">
	import Button from '$lib/ds/actions/Button.svelte';
	import ReleaseNotesDialog from '$lib/ds/chrome/ReleaseNotesDialog.svelte';

	let open = $state(false);
</script>

<Button label="What’s new" variant="secondary" onclick={() => (open = true)} />
<ReleaseNotesDialog
	bind:open
	releases={[
		{
			version: '0.1.7',
			date: '22 Nov 2026',
			url: 'https://github.com/triandamai/nomi-v2/releases/tag/v0.1.7',
			html: '<h2>What’s Changed</h2><ul><li>feat(workspace): device preview fallback, code tree, issues board</li><li>fix(chat): show action cards under the reply and drop repeated text</li></ul>',
		},
		{
			version: '0.1.6',
			date: '21 Nov 2026',
			html: '<h2>What’s Changed</h2><ul><li>fix(frontend-new): phone chats stay in their column; pages show while they load</li></ul>',
		},
	]}
/>
