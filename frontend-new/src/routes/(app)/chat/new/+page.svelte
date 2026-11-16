<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { m } from '$lib/paraglide/messages';
	import { mentionOptions, modelMenu, starters } from '$lib/adapters/chat';
	import { chatHeader } from '$lib/chat/header.svelte';
	import { startChat } from '$lib/chat/start';
	import { toasts } from '$lib/ds/overlays/toasts.svelte';
	import ChatComposer from '$lib/ds/thread/ChatComposer.svelte';
	import Starters from '$lib/ds/thread/Starters.svelte';
	import SuggestionRow from '$lib/ds/lists/SuggestionRow.svelte';
	import PhoneComposer from '$lib/ds/thread/PhoneComposer.svelte';
	import NavBar from '$lib/ds/mobile/NavBar.svelte';
	import Mascot from '$lib/ds/agents/Mascot.svelte';
	import { phoneView } from '$lib/phone.svelte';

	// A blank chat (New chat in the sidebar, ⌘K): laid out as a chat (Flow / Chat — Ask → think →
	// answer, ENb07: the thread column 720, the composer pinned at the bottom), with the starters
	// where the thread will be. Its first message makes the chat and opens it in place.
	// On a phone it's Flow / Mobile Chat — Start & answer, step 2: the nav bar (New chat, Nomi),
	// Nomi's mascot over the question at 22 and the line under it, centred, the starters 8 apart,
	// and the phone's composer.
	let { data } = $props();
	const guess = () => data.phone;
	const phone = phoneView(guess());

	const header = chatHeader();
	let draft = $state('');
	let field: HTMLTextAreaElement | undefined = $state();
	let modelKey = $state<string | null>(null);
	let busy = $state(false);

	const crew = $derived(mentionOptions(data.crew.rows));
	const models = $derived(modelMenu(data.models, modelKey ?? data.models.current?.key ?? null));
	const modelLabel = $derived(models.flatMap((group) => group.models).find((option) => option.chosen)?.label ?? data.models.current?.label);
	const picks = $derived(starters(data.crew.rows));

	async function start(text: string) {
		if (busy) return;
		busy = true;
		const started = await startChat(text, modelKey);
		busy = false;
		if (started.kind === 'opened') return void goto(started.href, { replaceState: true });
		draft = text;
		if (started.kind === 'offline') toasts.show({ kind: 'error', title: m.home_start_failed(), detail: m.home_start_offline() });
		else if (started.kind === 'signed_out') header.signedOutDraft = text;
		else toasts.show({ kind: 'error', title: m.home_start_failed(), detail: started.message || undefined });
	}

	onMount(() => {
		if (!matchMedia('(pointer: coarse)').matches) field?.focus();
	});
</script>

<svelte:head><title>{m.nav_new_chat()} · {m.app_name()}</title></svelte:head>

{#if phone.current}
	<div class="flex h-full flex-col">
		<NavBar title={m.nav_new_chat()} who={{ agent: 'nomi', name: 'Nomi' }} onback={() => void goto('/chats')} />
		<div class="min-h-0 flex-1 overflow-y-auto px-4 pt-4 pb-5">
			<div class="flex flex-col items-center gap-5 pt-12">
				<Mascot agent="nomi" size={32} />
				<h1 class="text-center text-phone-display font-semibold text-text">{m.chat_blank_title()}</h1>
				<p class="text-center text-body leading-comfy text-text-2">{m.home_intro()}</p>
				{#if picks.length > 0}
					<ul class="flex w-full flex-col gap-2 pt-2">
						{#each picks as pick, index (pick.label)}<li><SuggestionRow agent={pick.agent} label={pick.label} route={pick.route} onclick={() => start(picks[index].text)} /></li>{/each}
					</ul>
				{/if}
			</div>
		</div>
		<PhoneComposer bind:value={draft} bind:field placeholder={m.mobile_message()} {crew} model={modelLabel} onsend={start} />
	</div>
{:else}
<div class="flex h-full flex-col">
	<div class="min-h-0 flex-1 overflow-y-auto px-4 pt-6 lg:px-12 lg:pt-10">
		<div class="mx-auto flex min-h-full w-full max-w-180 flex-col justify-center gap-6 pb-8">
			<h1 class="text-display font-medium tracking-display text-text">{m.chat_blank_title()}</h1>
			{#if picks.length > 0}<Starters items={picks} onpick={(index) => start(picks[index].text)} />{/if}
		</div>
	</div>
	<div class="px-4 pb-4 lg:px-12 lg:pb-6">
		<div class="mx-auto w-full max-w-180">
			<ChatComposer
				bind:value={draft}
				bind:field
				placeholder={m.chat_placeholder()}
				{busy}
				pickerAbove
				{crew}
				model={modelLabel}
				{models}
				onsend={start}
				onmodel={(key) => (modelKey = key)}
			/>
		</div>
	</div>
</div>
{/if}
