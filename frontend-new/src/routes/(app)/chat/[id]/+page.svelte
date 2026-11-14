<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { chatHeader } from '$lib/chat/header.svelte';
	import ChatView from '$lib/chat/ChatView.svelte';

	// A chat (Flow / Chat): the thread and composer, shared with a project's workspace. The tab
	// follows a rename as it happens.
	let { data } = $props();

	const header = chatHeader();
	const title = $derived(header.shown(data.chat.id, 'title', data.chat.title));
</script>

<svelte:head><title>{title} · {m.app_name()}</title></svelte:head>

<ChatView {data} />
