<script lang="ts">
	import { page } from '$app/state';
	import ChatThread from '$lib/components/ChatThread.svelte';
	import ModelManager from '$lib/components/models/ModelManager.svelte';
	import ModelSelector from '$lib/components/models/ModelSelector.svelte';
	import { applyChanges, choiceFor, type ModelChange } from '$lib/modelPicker';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	// The model picker starts on the person's default. Nothing exists to save a pick to yet, so
	// it stays here until the first message, which carries it when it isn't the default.
	let modelGroups = $derived(data.modelOptions.groups);
	let pickedKey = $state<string | null>(null);
	const chosen = $derived((pickedKey ? choiceFor(modelGroups, pickedKey) : null) ?? data.modelOptions.current);
	const newChatModel = $derived(chosen && chosen.key !== data.modelOptions.current?.key ? chosen.key : null);
	let manageOpen = $state(false);

	const modelsSaved = (changes: ModelChange[]) => (modelGroups = applyChanges(modelGroups, changes));
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
	{newChatModel}
>
	{#snippet modelControl()}
		{#if modelGroups.length > 0 || chosen}
			<ModelSelector groups={modelGroups} selected={chosen} onselect={(key) => (pickedKey = key)} onmanage={() => (manageOpen = true)} />
		{/if}
	{/snippet}
</ChatThread>

<!-- Outside the thread: its composer is a form, and these hold forms of their own. -->
<ModelManager bind:open={manageOpen} groups={modelGroups} onsaved={modelsSaved} />
