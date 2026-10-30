<script lang="ts">
	import { Download, GitBranch, Globe, Pencil, Trash2 } from '@lucide/svelte';
	import MenuPanel from '$lib/ds/overlays/MenuPanel.svelte';
	import MenuRow from '$lib/ds/overlays/MenuRow.svelte';
	import Toast from '$lib/ds/overlays/Toast.svelte';
	import TooltipBubble from '$lib/ds/overlays/TooltipBubble.svelte';
	import DialogPanel from '$lib/ds/overlays/DialogPanel.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';
	import GallerySection from './GallerySection.svelte';

	// As the pen draws them, open: the menu with its items, the toasts and a tooltip, and a dialog
	// on the scrim. Menu, Tooltip and Dialog float these same panels in the app.
	const specimens = [
		{ caption: 'Menu Item / Default', item: { label: 'Export repo', icon: GitBranch, shortcut: '⌘E' } },
		{ caption: 'Menu Item / Hover', item: { label: 'Visit live site', icon: Globe, hover: true } },
		{ caption: 'Menu Item / Danger', item: { label: 'Delete project', icon: Trash2, danger: true } },
	];
</script>

<GallerySection
	title="Overlays"
	description="The only layers allowed a shadow. Surface fill, border-strong, one shadow level. Dialogs sit on nm-overlay."
>
	<div class="flex flex-wrap items-start gap-8 rounded-md bg-surface-2 p-8">
		<div class="flex flex-col gap-2">
			<MenuPanel
				id="gallery-menu"
				aria-label="Project actions"
				title="salah-daily-tracker"
				items={[
					{ label: 'Visit live site', icon: Globe, hover: true },
					{ label: 'Export repo', icon: GitBranch, shortcut: '⌘E' },
					{ label: 'Rename', icon: Pencil, shortcut: 'F2' },
					'divider',
					{ label: 'Delete project', icon: Trash2, danger: true },
				]}
			/>
			<span class="font-mono text-label text-text-3">Menu + items</span>
			{#each specimens as specimen (specimen.caption)}
				<div role="menu" aria-label={specimen.caption} class="w-55"><MenuRow item={specimen.item} /></div>
			{/each}
		</div>
		<div class="flex w-100 max-w-full flex-col items-start gap-3">
			<Toast kind="success" title="Deployed to salah-daily-tracker-0ab8f8.trian.space" detail="deploy #15 · built in 38s" action={{ label: 'Visit', onclick: () => {} }} ondismiss={() => {}} />
			<Toast kind="error" title="Deploy failed: build error in src/lib/pdf.ts" detail="Koda can try a fix, or open the log." action={{ label: 'Open log', onclick: () => {} }} ondismiss={() => {}} />
			<Toast kind="info" icon={Download} title="Repo exported: salah-daily-tracker.zip" detail="23 files · 184 KB" action={{ label: 'Show', onclick: () => {} }} ondismiss={() => {}} />
			<TooltipBubble label="New chat" shortcut="⌘K" />
		</div>
		<div class="flex h-75 min-w-0 flex-1 items-center justify-center rounded-md bg-overlay">
			<DialogPanel
				id="gallery-dialog"
				title="Delete salah-daily-tracker?"
				description="The live URL stops working and the database is removed. Your code stays in your account for 30 days; export it first if you want a copy."
			>
				<TextField label="Type the project name to confirm" placeholder="salah-daily-tracker" mono />
				{#snippet actions()}
					<Button label="Export repo first" variant="ghost" iconStart={Download} />
					<Button label="Cancel" variant="secondary" />
					<Button label="Delete project" variant="danger" iconStart={Trash2} />
				{/snippet}
			</DialogPanel>
		</div>
	</div>
</GallerySection>
