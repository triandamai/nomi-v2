<script lang="ts">
	import FileList from '$lib/components/ds/build/FileList.svelte';
	import PreviewBuilding from '$lib/components/ds/build/PreviewBuilding.svelte';
	import GallerySection from '../GallerySection.svelte';
	import GalleryBoard from '../GalleryBoard.svelte';
</script>

<GallerySection title="Building" description="The real step, the real file, the real clock. No percentage, no bar. The preview stays empty until there is something real to show.">
	<div class="grid items-start gap-x-8 gap-y-6 lg:grid-cols-2">
		<GalleryBoard title="File list" note="done · writing · queued" inline>
			<FileList
				total={9}
				files={[
					{ path: 'package.json', change: '+38', state: 'done' },
					{ path: 'svelte.config.js', change: '+14', state: 'done' },
					{ path: 'src/lib/db/schema.sql', change: '+46', state: 'done' },
					{ path: 'src/lib/db/client.ts', change: '+22', state: 'done' },
					{ path: 'src/routes/+layout.svelte', change: '+31', state: 'done' },
					{ path: 'src/routes/menu/+page.svelte', state: 'writing' },
					{ path: 'src/routes/order/+page.svelte', state: 'queued' },
					{ path: 'src/routes/order/+page.server.ts', state: 'queued' },
					{ path: 'src/routes/admin/orders/+page.svelte', state: 'queued' },
				]}
			/>
		</GalleryBoard>
		<GalleryBoard title="Preview before deploy" note="empty, honest" inline>
			<PreviewBuilding
				steps={[
					{ label: 'Set up project and database', state: 'done' },
					{ label: 'Menu page', state: 'running' },
					{ label: 'Pre-order form', state: 'waiting' },
					{ label: 'Orders view for Bu Siti', state: 'waiting' },
					{ label: 'Test on mobile', state: 'waiting' },
					{ label: 'Deploy to the live link', state: 'waiting' },
				]}
			/>
		</GalleryBoard>
	</div>
</GallerySection>
