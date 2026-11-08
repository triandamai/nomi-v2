<script lang="ts">
	// Above an issue chat's composer once the crew finished: is the issue done? "Yes" closes it in
	// Done (and the chat becomes read-only); "Not yet" leaves the composer to say what's missing,
	// which puts the crew back on it.
	import { m } from '$lib/paraglide/messages';
	import Button from '$lib/components/m3/Button.svelte';
	import Mascot from '$lib/components/ds/agents/Mascot.svelte';
	import { crewMember } from '$lib/issues';

	let {
		number,
		crewAgent,
		ondone,
	}: {
		number: number;
		crewAgent: string;
		/** Marks the issue done; throws when that fails. */
		ondone: () => Promise<void>;
	} = $props();

	const who = $derived(crewMember(crewAgent));
	let notYet = $state(false);
	let saving = $state(false);
	let error = $state<string | null>(null);

	async function done() {
		saving = true;
		try {
			await ondone();
			error = null;
		} catch {
			error = m.issue_followup_err();
		} finally {
			saving = false;
		}
	}
</script>

<div class="followup" role="group" aria-label={m.issue_followup_ask({ name: who.name, number })}>
	{#if notYet}
		<p class="followup__text">{m.issue_followup_not_yet_hint({ name: who.name })}</p>
	{:else}
		<p class="followup__text">
			{#if who.crew}<Mascot agent={who.crew} size={20} />{/if}
			{m.issue_followup_ask({ name: who.name, number })}
		</p>
		<div class="followup__actions">
			<Button type="button" variant="text" size="s" onclick={() => (notYet = true)}>{m.issue_followup_not_yet()}</Button>
			<Button type="button" variant="filled" size="s" disabled={saving} onclick={done}>{m.issue_followup_done()}</Button>
		</div>
	{/if}
	{#if error}<p class="followup__error" role="alert">{error}</p>{/if}
</div>

<style>
	.followup {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 6px 12px;
		margin: 0 0 8px;
		padding: 8px 8px 8px 12px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 12px;
		background: var(--md-sys-color-surface-container-low);
	}
	.followup__text {
		display: flex;
		flex: 1 1 220px;
		align-items: center;
		gap: 8px;
		margin: 0;
		color: var(--md-sys-color-on-surface);
		font-size: 0.875rem;
		font-weight: 600;
	}
	.followup__actions {
		display: flex;
		gap: 4px;
		margin-left: auto;
	}
	.followup__error {
		flex-basis: 100%;
		margin: 0;
		color: var(--md-sys-color-error);
		font-size: 0.8125rem;
	}
</style>
