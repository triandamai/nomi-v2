<script lang="ts">
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';
	import { afterNavigate } from '$app/navigation';
	import { page } from '$app/state';
	import RecentChats from '$lib/components/RecentChats.svelte';
	import { isRecentChatActive, loadRecentChats, refreshesAfterNavigation, type RecentsState } from '$lib/drawerRecents';
	import { onChatsChanged } from '$lib/userActivity';

	// Loads the drawer's Recents and keeps them current: after every navigation (a chat just
	// started or renamed shows up) and when the person picks another size in Preferences.
	let { limit, onnavigate }: { limit: number; onnavigate?: () => void } = $props();

	const EXPANDED_KEY = 'nomi:drawer-recents-expanded';
	let recents = $state<RecentsState>({ status: 'loading' });
	let expanded = $state(true);
	let loadedLimit: number | null = null;

	onMount(() => {
		try {
			expanded = localStorage.getItem(EXPANDED_KEY) !== 'false';
		} catch {
			expanded = true;
		}
	});

	function toggle() {
		expanded = !expanded;
		if (!browser) return;
		try {
			localStorage.setItem(EXPANDED_KEY, String(expanded));
		} catch {
			// Not remembered on this device; the section still opens and closes.
		}
	}

	async function refresh() {
		const requested = limit;
		const next = await loadRecentChats(fetch, requested);
		// Keep showing the last list when a background refresh fails, rather than an error.
		if (next.status === 'error' && recents.status === 'ready') return;
		recents = next;
		loadedLimit = requested;
	}

	function retry() {
		recents = { status: 'loading' };
		void refresh();
	}

	onMount(() => {
		void refresh();
		// A chat moved on, here or on another of the person's devices: it goes to the top.
		return onChatsChanged(() => void refresh());
	});
	afterNavigate(({ from, to }) => {
		if (refreshesAfterNavigation(from, to)) void refresh();
	});

	$effect(() => {
		if (loadedLimit !== null && limit !== loadedLimit) void refresh();
	});

	const activeId = $derived(recents.status === 'ready' ? (recents.chats.find((chat) => isRecentChatActive(page.url.pathname, chat.id))?.id ?? null) : null);
</script>

<RecentChats state={recents} {expanded} {activeId} ontoggle={toggle} {onnavigate} onretry={retry} />
