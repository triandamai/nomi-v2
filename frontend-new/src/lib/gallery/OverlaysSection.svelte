<script lang="ts">
	import { Download, GitBranch, Globe, Pencil, Plus, Trash2 } from '@lucide/svelte';
	import Menu from '$lib/ds/overlays/Menu.svelte';
	import Toast from '$lib/ds/overlays/Toast.svelte';
	import Tooltip from '$lib/ds/overlays/Tooltip.svelte';
	import Dialog from '$lib/ds/overlays/Dialog.svelte';
	import ToastRegion from '$lib/ds/overlays/ToastRegion.svelte';
	import { toasts } from '$lib/ds/overlays/toasts.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import IconButton from '$lib/ds/actions/IconButton.svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';
	import GallerySection from './GallerySection.svelte';

	let dialogOpen = $state(false);
</script>

{#snippet caption(text: string)}<span class="font-mono text-label text-text-3">{text}</span>{/snippet}

<GallerySection
	title="Overlays"
	description="The only layers allowed a shadow. Surface fill, border-strong, one shadow level. Dialogs sit on nm-overlay."
>
	<div class="flex flex-wrap items-start gap-8 rounded-md border border-border p-5">
		<div class="flex flex-col items-start gap-2">
			<Menu
				title="salah-daily-tracker"
				items={[
					{ label: 'Export repo', icon: GitBranch, shortcut: '⌘E' },
					{ label: 'Visit live site', icon: Globe },
					{ label: 'Rename', icon: Pencil, shortcut: 'F2' },
					'divider',
					{ label: 'Delete project', icon: Trash2, danger: true },
				]}
			>
				{#snippet trigger(props)}<Button label="Project actions" variant="secondary" {...props} />{/snippet}
			</Menu>
			{@render caption('Menu + items')}
		</div>
		<div class="flex flex-col items-start gap-2">
			<Tooltip label="New chat" shortcut="⌘K">
				{#snippet children(props)}<IconButton icon={Plus} label="New chat" {...props} />{/snippet}
			</Tooltip>
			{@render caption('Tooltip (hover or focus)')}
		</div>
		<div class="flex flex-col items-start gap-2">
			<Button label="Delete project" variant="danger" iconStart={Trash2} onclick={() => (dialogOpen = true)} />
			{@render caption('Dialog')}
		</div>
		<div class="flex flex-col items-start gap-2">
			<Button label="Show a toast" variant="secondary" onclick={() => toasts.show({ kind: 'info', title: 'Reply deleted', action: { label: 'Undo', onclick: () => {} }, duration: 10_000 })} />
			{@render caption('Toast region')}
		</div>
	</div>

	<div class="flex flex-col gap-3 rounded-md border border-border p-5">
		<Toast kind="success" title="Deployed to salah-daily-tracker-0ab8f8.trian.space" detail="deploy #15 · built in 38s" action={{ label: 'Visit', onclick: () => {} }} ondismiss={() => {}} />
		<Toast kind="error" title="Deploy failed: build error in src/lib/pdf.ts" detail="Koda can try a fix, or open the log." action={{ label: 'Open log', onclick: () => {} }} ondismiss={() => {}} />
		<Toast kind="info" icon={Download} title="Repo exported: salah-daily-tracker.zip" detail="23 files · 184 KB" action={{ label: 'Show', onclick: () => {} }} ondismiss={() => {}} />
		{@render caption('Toast / Success · Error · Info')}
	</div>
</GallerySection>

<Dialog
	bind:open={dialogOpen}
	title="Delete salah-daily-tracker?"
	description="The live URL stops working and the database is removed. Your code stays in your account for 30 days; export it first if you want a copy."
>
	<TextField label="Type the project name to confirm" placeholder="salah-daily-tracker" mono />
	{#snippet actions()}
		<Button label="Export repo first" variant="ghost" iconStart={Download} />
		<Button label="Cancel" variant="secondary" onclick={() => (dialogOpen = false)} />
		<Button label="Delete project" variant="danger" />
	{/snippet}
</Dialog>
<ToastRegion />
