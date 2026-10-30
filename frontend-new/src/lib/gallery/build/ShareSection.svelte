<script lang="ts">
	import BuildSharePanel from '$lib/ds/build/BuildSharePanel.svelte';
	import ExportPanel from '$lib/ds/build/ExportPanel.svelte';
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
</script>

<GallerySection title="Share & export" description="Two dialogs reached from the live card. Export is never behind a plan: it takes the same number of clicks as upgrading, and works on Free.">
	<div class="grid items-start gap-x-8 gap-y-6 lg:grid-cols-2">
		<GalleryBoard title="Share" note="copy · QR · WhatsApp · public or unlisted" inline>
			<BuildSharePanel {url} {preview} oneditpreview={noop} oninvite={noop} onmore={noop} onclose={noop} ondone={noop} />
		</GalleryBoard>
		<GalleryBoard title="Export" note="everything included · two equal ways out" inline>
			<ExportPanel {...exported} onpush={noop} ondownload={noop} onclose={noop} />
		</GalleryBoard>
	</div>
</GallerySection>

