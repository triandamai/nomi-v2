<script lang="ts">
	import BuildSharePanel from '$lib/ds/build/BuildSharePanel.svelte';
	import BuildShareDialog from '$lib/ds/build/BuildShareDialog.svelte';
	import ExportPanel from '$lib/ds/build/ExportPanel.svelte';
	import ExportDialog from '$lib/ds/build/ExportDialog.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import GallerySection from '../GallerySection.svelte';
	import GalleryBoard from '../GalleryBoard.svelte';

	const noop = () => {};
	const url = 'https://warung-kopi-menu-3f9a12.trian.space';
	const preview = { title: 'Warung Kopi Bu Siti', description: 'Menu and pre-order for pickup' };
	const exported = {
		project: 'warung-kopi-menu',
		repo: '9 files · git history · main@4be1c07',
		database: 'warung-kopi-menu.sql · menu_items, orders',
		env: 'DATABASE_URL · PUBLIC_SITE_URL · ADMIN_PIN',
	};
	let sharing = $state(false);
	let exporting = $state(false);
</script>

<GallerySection title="Share & export" description="Two dialogs reached from the live card. Export is never behind a plan: it takes the same number of clicks as upgrading, and works on Free.">
	<div class="grid items-start gap-x-8 gap-y-6 lg:grid-cols-2">
		<GalleryBoard title="Share" note="copy · QR · WhatsApp · public or unlisted" inline>
			<div class="flex flex-col items-start gap-4">
				<BuildSharePanel {url} {preview} oneditpreview={noop} oninvite={noop} onmore={noop} onclose={noop} ondone={noop} />
				<Button label="Open share" variant="secondary" onclick={() => (sharing = true)} />
			</div>
		</GalleryBoard>
		<GalleryBoard title="Export" note="everything included · two equal ways out" inline>
			<div class="flex flex-col items-start gap-4">
				<ExportPanel {...exported} onpush={noop} ondownload={noop} onclose={noop} />
				<Button label="Open export" variant="secondary" onclick={() => (exporting = true)} />
			</div>
		</GalleryBoard>
	</div>
</GallerySection>

<BuildShareDialog bind:open={sharing} {url} {preview} oninvite={noop} onmore={noop} />
<ExportDialog bind:open={exporting} {...exported} onpush={noop} ondownload={noop} />
