<script lang="ts">
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
	thinkingLevel="medium"
/>
