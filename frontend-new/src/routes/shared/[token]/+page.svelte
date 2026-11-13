<script lang="ts">
	import { onMount } from 'svelte';
	import { ArrowRight, Link2Off, LockKeyhole, LogIn } from '@lucide/svelte';
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { crewIdOf } from '$lib/adapters/chat';
	import { clock } from '$lib/chat/dates';
	import Mascot from '$lib/ds/agents/Mascot.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import Icon from '$lib/ds/foundations/Icon.svelte';
	import Tag from '$lib/ds/status/Tag.svelte';
	import Prose from '$lib/ds/thread/Prose.svelte';

	// Flow / Chat — Shared link (b8Flw8): what people see when someone shares a chat or a reply. A
	// read-only page with no sign-in (unless the link is for signed-in people), masked if the person
	// sharing chose to: the top bar (Nomi, Sign in, Try Nomi), the share's meta line, the messages in
	// a 720 column, and a footer. A link that's expired, turned off or unknown shows one screen in the
	// pen's expired/revoked layout, since the backend tells them apart for nobody (AGENTS.md §9).
	let { data } = $props();

	const language = getLocale();
	let timeZone = $state<string | undefined>();
	onMount(() => (timeZone = Intl.DateTimeFormat().resolvedOptions().timeZone));

	const signIn = $derived(`/sign-in?redirect_to=${encodeURIComponent(page.url.pathname)}`);
	const daysLeft = (iso: string) => Math.max(1, Math.ceil((Date.parse(iso) - Date.now()) / 86_400_000));
</script>

<svelte:head>
	<title>{data.state === 'open' ? (data.chat.title ?? m.shared_untitled()) : m.shared_gone_title()} · {m.app_name()}</title>
	<meta name="robots" content="noindex" />
</svelte:head>

<div class="flex min-h-screen flex-col bg-bg">
	<header class="flex h-14 items-center justify-between border-b border-border px-4 lg:px-8">
		<a href="/" class="flex items-center gap-2 rounded-xs outline-offset-2"><Mascot agent="nomi" size={20} /><span class="text-body font-medium text-text">Nomi</span></a>
		<span class="flex items-center gap-2">
			<Button label={m.shared_sign_in()} variant="ghost" iconStart={LogIn} href={signIn} />
			<Button label={m.shared_try()} iconEnd={ArrowRight} href="/register" />
		</span>
	</header>

	<main class="flex flex-1 flex-col items-center px-4 py-12">
		{#if data.state === 'open'}
			{@const chat = data.chat}
			<div class="flex w-full max-w-180 flex-col gap-7">
				<div class="flex flex-wrap items-center gap-x-2.5 gap-y-1 border-b border-border pb-4">
					<Tag kind="neutral" label={m.shared_read_only()} />
					{#if chat.title}<span class="text-caption text-text-2">{chat.title}</span>{/if}
					<span class="font-mono text-label text-text-3">
						{[chat.expiresAt ? m.shared_expires({ days: String(daysLeft(chat.expiresAt)) }) : m.shared_no_expiry(), chat.masked ? m.shared_masked() : ''].filter(Boolean).join(' · ')}
					</span>
				</div>
				{#each chat.messages as message, index (index)}
					{#if message.sender === 'user'}
						<div class="flex justify-end"><div class="max-w-xl rounded-md bg-surface-2 px-3.5 py-2 text-body text-text"><Prose html={message.html} /></div></div>
					{:else}
						<article class="flex flex-col gap-4" aria-label={message.by ?? 'Nomi'}>
							<span class="flex items-center gap-2">
								<Mascot agent={crewIdOf(message.by)} size={20} />
								<span class="text-ui font-medium text-text">{message.by ?? 'Nomi'}</span>
								<span class="font-mono text-label text-text-3">{clock(message.at, timeZone)}</span>
							</span>
							<Prose html={message.html} />
						</article>
					{/if}
				{/each}
			</div>
		{:else}
			<div class="flex w-full max-w-120 flex-col items-start gap-3">
				<span class="inline-flex size-11 items-center justify-center rounded-md bg-surface-2 text-text-2"><Icon icon={data.state === 'signin' ? LockKeyhole : Link2Off} size={20} /></span>
				<h1 class="text-heading font-medium text-text">{data.state === 'signin' ? m.shared_signin_title() : m.shared_gone_title()}</h1>
				<p class="text-ui leading-normal text-text-2">{data.state === 'signin' ? m.shared_signin_body() : m.shared_gone_body()}</p>
				<span class="flex flex-wrap gap-2 pt-2">
					{#if data.state === 'signin'}<Button label={m.shared_sign_in()} variant="secondary" iconStart={LogIn} href={signIn} />{/if}
					<Button label={m.shared_try()} iconEnd={ArrowRight} href="/register" />
				</span>
			</div>
		{/if}
	</main>

	<footer class="flex items-center border-t border-border px-4 pt-3.5 pb-3.25 lg:px-8">
		<span class="font-mono text-label text-text-3">{m.shared_made_with()}</span>
	</footer>
</div>
