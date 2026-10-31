<!-- Writing: done, the one being written now, and the ones still to come. -->
<script lang="ts">
	import FileList from '$lib/ds/build/FileList.svelte';
</script>

<div class="w-full max-w-110">
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
</div>
