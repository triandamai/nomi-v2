<script lang="ts">
	import { page } from '$app/state';
	import ChatThread from '$lib/components/ChatThread.svelte';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();
</script>

<ChatThread
	sessionId=""
	unsaved
	title={data.title}
	messages={[]}
	agentActivity={[]}
	sendError={form?.error ?? null}
	glass={page.data.preferences?.animated_backdrop !== false}
	thinkingLevel="medium"
/>
