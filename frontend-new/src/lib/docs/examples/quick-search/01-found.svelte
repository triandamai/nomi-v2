<!-- What “belanja” found: a chat and a file. -->
<script lang="ts">
	import QuickSearch from '$lib/ds/chrome/QuickSearch.svelte';
</script>

<div class="flex min-h-80 w-full justify-end">
	<QuickSearch
		value="belanja"
		shortcut="⌘K"
		groups={[
			{ kind: 'chat', rows: [{ id: 'c1', title: 'Budget belanja Oktober', snippet: '…belanja bulanan naik 10% dari September…', meta: '2h ago', href: '#' }] },
			{ kind: 'file', rows: [{ id: 'f1', title: 'src/routes/belanja/+page.svelte', meta: '1d ago', href: '#' }] },
		]}
		labels={{ field: 'Search chats, memory, files', groups: { chat: 'Chats', memory: 'Memory', file: 'Files' }, empty: 'Nothing found', searching: 'Searching…', failed: 'Search isn’t answering.' }}
	/>
</div>
