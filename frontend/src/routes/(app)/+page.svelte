<script lang="ts">
	import { enhance } from '$app/forms';
	import { page } from '$app/state';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import Chip from '$lib/components/m3/Chip.svelte';
	import SendButton from '$lib/components/m3/SendButton.svelte';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	let text = $state('');
	let submitting = $state(false);
	let input: HTMLTextAreaElement | undefined = $state();

	const name = $derived(data.profile.display_name?.split(' ')[0] ?? null);

	// Greeting follows the user's stored timezone, not the server's, so SSR and the browser agree.
	const greeting = $derived.by(() => {
		let hour = new Date().getHours();
		try {
			hour = Number(
				new Intl.DateTimeFormat('en-US', { hour: 'numeric', hourCycle: 'h23', timeZone: data.preferences.timezone }).format(
					new Date(),
				),
			);
		} catch {
			// Unknown timezone string — fall back to the runtime's local hour.
		}
		if (hour < 5) return 'Still up';
		if (hour < 12) return 'Morning';
		if (hour < 18) return 'Afternoon';
		return 'Evening';
	});

	const activeCount = $derived(data.recentSessions.filter((s) => s.agent_active).length);

	const SUGGESTIONS = [
		'Summarize this week’s spending',
		'Plan my Saturday',
		'Start a small website project',
		'Remind me to stretch at 4pm',
	];

	const CREW = [
		{ agent: 'nomi', name: 'Nomi', role: 'Talks with you and routes the work' },
		{ agent: 'money', name: 'Money', role: 'Transactions, budgets, subscriptions' },
		{ agent: 'coding', name: 'Coding', role: 'Builds and edits project files' },
		{ agent: 'planning', name: 'Planning', role: 'Plans, to-dos and reminders' },
		{ agent: 'personality', name: 'Personality', role: 'Keeps Nomi sounding like you want' },
	];

	function useSuggestion(suggestion: string) {
		text = suggestion;
		input?.focus();
	}

	function onKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter' && !event.shiftKey && !event.isComposing) {
			event.preventDefault();
			if (text.trim()) (event.currentTarget as HTMLTextAreaElement).form?.requestSubmit();
		}
	}

	function timeAgo(iso: string): string {
		const minutes = Math.floor((Date.now() - new Date(iso).getTime()) / 60000);
		if (minutes < 1) return 'now';
		if (minutes < 60) return `${minutes}m`;
		const hours = Math.floor(minutes / 60);
		if (hours < 24) return `${hours}h`;
		return `${Math.floor(hours / 24)}d`;
	}
</script>

<div class="home">
	<div class="home__inner">
		{#if page.url.searchParams.get('error') === 'forbidden'}
			<p class="md-body-medium home__error" role="alert">That account does not have admin access.</p>
		{/if}

		<section class="home__hero">
			<div class="home__lead">
				<h1 class="md-display-large home__title">
					{greeting}{name ? `, ${name}` : ''}.
					<span class="home__title-sub">
						{activeCount > 0
							? `Your crew is working in ${activeCount} ${activeCount === 1 ? 'chat' : 'chats'}.`
							: 'What should we get done?'}
					</span>
				</h1>

				<form
					method="POST"
					action="?/newChat"
					class="composer"
					use:enhance={() => {
						submitting = true;
						return async ({ update }) => {
							await update();
							submitting = false;
						};
					}}
				>
					<label for="home-ask" class="sr-only">Message Nomi</label>
					<textarea
						id="home-ask"
						bind:this={input}
						bind:value={text}
						name="text"
						rows="2"
						placeholder="Ask Nomi, or hand something to the crew…"
						class="composer__input"
						onkeydown={onKeydown}
					></textarea>
					<div class="composer__bar">
						<span class="nomi-meta">Enter to send · Shift+Enter for a new line</span>
						<SendButton working={submitting} disabled={!text.trim()} label="Start chat" />
					</div>
				</form>
				{#if form?.error}
					<p class="md-body-medium home__error" role="alert">{form.error}</p>
				{/if}

				<div class="home__chips">
					{#each SUGGESTIONS as suggestion (suggestion)}
						<Chip variant="suggestion" onclick={() => useSuggestion(suggestion)}>{suggestion}</Chip>
					{/each}
				</div>
			</div>

			<section class="crew" aria-labelledby="crew-heading">
				<h2 id="crew-heading" class="crew__title">Your crew</h2>
				<ul class="crew__list">
					{#each CREW as member (member.agent)}
						<li class="crew__item">
							<AgentShape agent={member.agent} size={44} face={member.agent === 'nomi'} />
							<span class="crew__text">
								<span class="crew__name">{member.name}</span>
								<span class="crew__role">{member.role}</span>
							</span>
						</li>
					{/each}
				</ul>
			</section>
		</section>

		<section class="recent" aria-labelledby="recent-heading">
			<div class="recent__head">
				<h2 id="recent-heading" class="md-title-large" style="font-weight: 700">Recent</h2>
				<a href="/chats" class="recent__all">All chats</a>
			</div>
			{#if data.recentSessions.length === 0}
				<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">
					Nothing yet — your chats will show up here.
				</p>
			{:else}
				<ul class="recent__grid">
					{#each data.recentSessions as session (session.id)}
						<li>
							<a
								class="recent__card"
								href={session.project_id ? `/projects/session/${session.id}` : `/chat/${session.id}`}
							>
								<span class="recent__top">
									<AgentShape size={28} working={session.agent_active} />
									<span class="nomi-meta">{session.agent_active ? 'Working' : timeAgo(session.updated_at)}</span>
								</span>
								<span class="recent__name">{session.title ?? 'New chat'}</span>
								<span class="recent__preview">{session.last_message?.content ?? 'No messages yet'}</span>
							</a>
						</li>
					{/each}
				</ul>
			{/if}
		</section>
	</div>
</div>

<style>
	.home {
		height: 100%;
		overflow-y: auto;
		padding: 32px clamp(16px, 4vw, 56px) 56px;
		box-sizing: border-box;
	}
	.home__inner {
		max-width: 1180px;
		margin: 0 auto;
		display: flex;
		flex-direction: column;
		gap: 48px;
	}
	.home__error {
		color: var(--md-sys-color-error);
		margin: 0;
	}

	.home__hero {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(min(100%, 380px), 1fr));
		gap: 28px;
		align-items: start;
	}
	.home__lead {
		display: flex;
		flex-direction: column;
		gap: 24px;
		min-width: 0;
		grid-column: span 1;
	}
	@media (min-width: 1100px) {
		.home__hero {
			grid-template-columns: minmax(0, 1.5fr) minmax(0, 1fr);
		}
	}
	.home__title {
		margin: 0;
		color: var(--md-sys-color-on-surface);
	}
	.home__title-sub {
		display: block;
		margin-top: 0.15em;
		font-size: 0.5em;
		line-height: 1.1;
		letter-spacing: -0.02em;
		font-weight: 650;
		color: var(--md-sys-color-on-surface-variant);
	}

	.composer {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 20px 14px 14px 24px;
		border-radius: var(--md-sys-shape-corner-extra-large-increased);
		background: var(--md-sys-color-surface-container-lowest);
		box-shadow:
			0 1px 0 var(--md-sys-color-outline-variant),
			0 18px 40px -28px color-mix(in srgb, var(--md-sys-color-on-surface) 45%, transparent);
	}
	.composer__input {
		border: none;
		resize: none;
		outline: none;
		background: transparent;
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-sys-typescale-body-large-font);
		font-size: 1.125rem;
		line-height: 1.5;
		padding-right: 10px;
	}
	.composer__input::placeholder {
		color: var(--md-sys-color-on-surface-variant);
	}
	.composer__bar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
	}
	.composer__bar .nomi-meta {
		text-transform: none;
		letter-spacing: 0.02em;
	}
	@media (max-width: 640px) {
		.composer__bar .nomi-meta {
			visibility: hidden;
		}
	}
	.home__chips {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}

	.crew {
		padding: 28px;
		border-radius: 40px 40px 40px 12px;
		background: var(--nomi-color-stage);
		color: var(--nomi-color-on-stage);
		min-width: 0;
	}
	.crew__title {
		margin: 0 0 12px;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.75rem;
		font-weight: 700;
		letter-spacing: -0.02em;
	}
	.crew__list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.crew__item {
		display: flex;
		align-items: center;
		gap: 14px;
		padding: 10px 0;
	}
	.crew__text {
		display: flex;
		flex-direction: column;
		min-width: 0;
	}
	.crew__name {
		font-weight: 650;
		font-size: 1rem;
	}
	.crew__role {
		font-size: 0.875rem;
		opacity: 0.75;
	}

	.recent__head {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		margin-bottom: 16px;
	}
	.recent__head h2 {
		margin: 0;
	}
	.recent__all {
		color: var(--md-sys-color-primary);
		font-weight: 600;
		text-decoration: none;
		padding: 8px 4px;
	}
	.recent__grid {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(min(100%, 260px), 1fr));
		gap: 16px;
	}
	.recent__card {
		display: flex;
		flex-direction: column;
		gap: 8px;
		height: 100%;
		box-sizing: border-box;
		padding: 18px 20px 20px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
		text-decoration: none;
		transition:
			border-radius var(--nomi-motion-spatial-fast),
			background-color var(--nomi-motion-effects-fast);
	}
	.recent__card:hover {
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
	}
	.recent__top {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}
	.recent__name {
		font-weight: 650;
		font-size: 1rem;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.recent__preview {
		font-size: 0.875rem;
		opacity: 0.8;
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}
</style>
