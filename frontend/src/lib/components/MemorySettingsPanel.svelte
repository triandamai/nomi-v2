<script lang="ts">
	// Settings → Memory: the "You" profile Nomi keeps (editable), and what Nomi may do with memory
	// (Learn from chats, Keep a profile, Search past chats, and the budgets with their use).
	import { m } from '$lib/paraglide/messages';
	import { enhance } from '$app/forms';
	import { untrack } from 'svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import Switch from '$lib/components/m3/Switch.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import { timeAgo } from '$lib/memory';
	import { budgetShare, MEMORY_BUDGET_RANGE, PROFILE_BUDGET_RANGE } from '$lib/memorySettings';
	import type { MemoryProfile, MemorySettings } from '$lib/types';

	let {
		settings,
		profile,
		settingsError = null,
		settingsSaved = false,
		profileError = null,
		profileSaved = false,
		draftText = null,
	}: {
		settings: MemorySettings;
		profile: MemoryProfile | null;
		settingsError?: string | null;
		settingsSaved?: boolean;
		profileError?: string | null;
		profileSaved?: boolean;
		/** What the person typed, kept when saving failed. */
		draftText?: string | null;
	} = $props();

	let learn = $state(untrack(() => settings.learn_from_chats));
	let keepProfile = $state(untrack(() => settings.keep_profile));
	let pastChats = $state(untrack(() => settings.search_past_chats));
	let memoryBudget = $state(untrack(() => String(settings.memory_budget)));
	let profileBudget = $state(untrack(() => String(settings.profile_budget)));
	let profileText = $state(untrack(() => draftText ?? profile?.text ?? ''));
	let saving = $state(false);

	const memoryShare = $derived(budgetShare(settings.memory_used, settings.memory_budget));
	const profileShare = $derived(budgetShare(settings.profile_used, settings.profile_budget));
	const uid = $props.id();

	const saveWith = () => {
		saving = true;
		return async ({ update }: { update: (opts?: { reset?: boolean }) => Promise<void> }) => {
			await update({ reset: false });
			saving = false;
		};
	};
</script>

<section class="panel" aria-labelledby="{uid}-profile">
	<h2 id="{uid}-profile" class="md-title-large panel__title">{m.mem_profile_title()}</h2>
	<p class="panel__lede">{m.mem_profile_lede()}</p>
	{#if !settings.keep_profile}
		<p class="panel__lede">{m.mem_profile_off()}</p>
	{/if}
	<form method="POST" action="?/profile" class="panel__form" use:enhance={saveWith}>
		<label class="profile">
			<span class="sr-only">{m.mem_profile_title()}</span>
			<textarea name="text" rows="6" bind:value={profileText} placeholder={m.mem_profile_empty()}></textarea>
		</label>
		<p class="panel__meta">
			{m.mem_set_use({ used: settings.profile_used, budget: settings.profile_budget })}
			{#if profile?.updated_at}· {m.mem_profile_updated({ when: timeAgo(profile.updated_at) })}{/if}
		</p>
		{#if profileError}
			<p class="panel__error" role="alert">{profileError}</p>
		{:else if profileSaved}
			<p class="panel__meta" role="status">{m.mem_profile_saved()}</p>
		{/if}
		<Button variant="tonal" size="s" type="submit" disabled={saving}>{m.mem_profile_save()}</Button>
	</form>
</section>

<section class="panel" aria-labelledby="{uid}-settings">
	<h2 id="{uid}-settings" class="md-title-large panel__title">{m.mem_settings_title()}</h2>
	<form method="POST" action="?/settings" class="panel__form" use:enhance={saveWith}>
		<ul class="switches">
			<li>
				<span class="switches__text">
					<span class="switches__name" id="{uid}-learn">{m.mem_set_learn()}</span>
					<span class="panel__meta">{m.mem_set_learn_desc()}</span>
				</span>
				<Switch name="learn_from_chats" bind:checked={learn} aria-label={m.mem_set_learn()} />
			</li>
			<li>
				<span class="switches__text">
					<span class="switches__name">{m.mem_set_profile()}</span>
					<span class="panel__meta">{m.mem_set_profile_desc()}</span>
				</span>
				<Switch name="keep_profile" bind:checked={keepProfile} aria-label={m.mem_set_profile()} />
			</li>
			<li>
				<span class="switches__text">
					<span class="switches__name">{m.mem_set_past()}</span>
					<span class="panel__meta">{m.mem_set_past_desc()}</span>
				</span>
				<Switch name="search_past_chats" bind:checked={pastChats} aria-label={m.mem_set_past()} />
			</li>
		</ul>

		<div class="budget">
			<TextField
				label={m.mem_set_memory_budget()}
				name="memory_budget"
				type="number"
				min={MEMORY_BUDGET_RANGE.min}
				max={MEMORY_BUDGET_RANGE.max}
				bind:value={memoryBudget}
			/>
			<div class="meter" role="meter" aria-valuemin="0" aria-valuemax="100" aria-valuenow={memoryShare} aria-label={m.mem_set_memory_budget()}>
				<span class="meter__fill" style:width="{memoryShare}%"></span>
			</div>
			<p class="panel__meta">{m.mem_set_use({ used: settings.memory_used, budget: settings.memory_budget })} · {m.mem_set_budget_hint()}</p>
		</div>
		<div class="budget">
			<TextField
				label={m.mem_set_profile_budget()}
				name="profile_budget"
				type="number"
				min={PROFILE_BUDGET_RANGE.min}
				max={PROFILE_BUDGET_RANGE.max}
				bind:value={profileBudget}
			/>
			<div class="meter" role="meter" aria-valuemin="0" aria-valuemax="100" aria-valuenow={profileShare} aria-label={m.mem_set_profile_budget()}>
				<span class="meter__fill" style:width="{profileShare}%"></span>
			</div>
			<p class="panel__meta">{m.mem_set_use({ used: settings.profile_used, budget: settings.profile_budget })}</p>
		</div>

		{#if settingsError}
			<p class="panel__error" role="alert">{settingsError}</p>
		{:else if settingsSaved}
			<p class="panel__meta" role="status">{m.mem_set_saved()}</p>
		{/if}
		<Button variant="filled" size="s" type="submit" disabled={saving}>{m.mem_set_save()}</Button>
	</form>
</section>

<style>
	.panel {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 20px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
	}
	.panel__title {
		margin: 0;
		color: var(--md-sys-color-on-surface);
	}
	.panel__lede,
	.panel__meta {
		margin: 0;
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.panel__error {
		margin: 0;
		color: var(--md-sys-color-error);
	}
	.panel__form {
		display: flex;
		flex-direction: column;
		align-items: stretch;
		gap: 12px;
	}
	.panel__form > :global(button) {
		align-self: flex-start;
	}
	.profile textarea {
		box-sizing: border-box;
		width: 100%;
		padding: 12px 14px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-low);
		color: var(--md-sys-color-on-surface);
		font: inherit;
		resize: vertical;
	}
	.profile textarea:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 1px;
	}
	.switches {
		display: flex;
		flex-direction: column;
		gap: 14px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.switches li {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 16px;
	}
	.switches__text {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.switches__name {
		font-weight: 600;
		color: var(--md-sys-color-on-surface);
	}
	.budget {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.meter {
		height: 6px;
		overflow: hidden;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container-highest);
	}
	.meter__fill {
		display: block;
		height: 100%;
		background: var(--md-sys-color-primary);
	}
	.sr-only {
		position: absolute;
		width: 1px;
		height: 1px;
		overflow: hidden;
		clip: rect(0 0 0 0);
		white-space: nowrap;
	}
</style>
