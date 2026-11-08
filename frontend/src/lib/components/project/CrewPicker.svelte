<script lang="ts">
	// Picks the crew member an issue goes to: type a name or what they do, and pick from the
	// matches (arrow keys and Enter, or tap). Shows who's picked with their mascot.
	import Mascot from '$lib/components/ds/agents/Mascot.svelte';
	import { m } from '$lib/paraglide/messages';
	import { ISSUE_CREW, crewMatches } from '$lib/issues';

	let {
		value = $bindable(),
		label,
		hint,
		role,
	}: {
		/** The picked crew member's agent type. */
		value: string;
		label: string;
		hint?: string;
		/** What a crew member does, for the list and the search. */
		role: (agent: string) => string;
	} = $props();

	const uid = $props.id();
	let query = $state('');
	let open = $state(false);
	let highlighted = $state(0);
	let input: HTMLInputElement | undefined = $state();

	const picked = $derived(ISSUE_CREW.find((member) => member.agent === value) ?? ISSUE_CREW[0]);
	const matches = $derived(crewMatches(query, role));

	function choose(agent: string) {
		value = agent;
		query = '';
		open = false;
	}

	function onkeydown(event: KeyboardEvent) {
		if (event.key === 'ArrowDown') {
			open = true;
			highlighted = Math.min(highlighted + 1, matches.length - 1);
		} else if (event.key === 'ArrowUp') {
			highlighted = Math.max(highlighted - 1, 0);
		} else if (event.key === 'Enter' && open && matches.length > 0) {
			choose(matches[highlighted].agent);
		} else if (event.key === 'Escape' && open) {
			open = false;
			query = '';
		} else {
			return;
		}
		event.preventDefault();
		event.stopPropagation();
	}
</script>

<div class="crew-picker">
	<label class="crew-picker__label" for="{uid}-input">{label}</label>
	<div class="crew-picker__box">
		<Mascot agent={picked.crew} size={20} />
		<input
			bind:this={input}
			id="{uid}-input"
			type="text"
			role="combobox"
			autocomplete="off"
			aria-autocomplete="list"
			aria-expanded={open}
			aria-controls="{uid}-list"
			aria-activedescendant={open && matches.length > 0 ? `${uid}-opt-${highlighted}` : undefined}
			placeholder={picked.name}
			bind:value={query}
			oninput={() => {
				open = true;
				highlighted = 0;
			}}
			onfocus={() => (open = true)}
			onblur={() => {
				open = false;
				query = '';
			}}
			{onkeydown}
		/>
		<span class="crew-picker__role">{role(picked.agent)}</span>
	</div>
	{#if open}
		<ul class="crew-picker__list" id="{uid}-list" role="listbox" aria-label={label}>
			{#each matches as member, index (member.agent)}
				<!-- The keyboard drives the list from the input (a combobox), so the options take only the pointer. -->
				<!-- svelte-ignore a11y_click_events_have_key_events -->
				<li
					id="{uid}-opt-{index}"
					role="option"
					class="crew-picker__option"
					aria-selected={member.agent === value}
					class:crew-picker__option--hi={index === highlighted}
					onmousedown={(event) => event.preventDefault()}
					onmouseenter={() => (highlighted = index)}
					onclick={() => {
						choose(member.agent);
						input?.blur();
					}}
				>
					<Mascot agent={member.crew} size={32} />
					<span class="crew-picker__text">
						<span class="crew-picker__name">{member.name}</span>
						<span class="crew-picker__what">{role(member.agent)}</span>
					</span>
				</li>
			{:else}
				<li class="crew-picker__none">{m.issues_crew_none()}</li>
			{/each}
		</ul>
	{/if}
	{#if hint}<span class="crew-picker__hint">{hint}</span>{/if}
</div>

<style>
	.crew-picker {
		position: relative;
		display: flex;
		flex-direction: column;
		gap: 4px;
		min-width: 0;
	}
	.crew-picker__label,
	.crew-picker__hint {
		font-size: var(--md-sys-typescale-body-small-size);
		color: var(--md-sys-color-on-surface-variant);
	}
	.crew-picker__box {
		display: flex;
		align-items: center;
		gap: 10px;
		height: 52px;
		padding: 0 14px;
		box-sizing: border-box;
		border: 1.5px solid var(--md-sys-color-outline);
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface);
		transition: background-color var(--nomi-motion-effects-fast);
	}
	.crew-picker__box:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 4%, var(--md-sys-color-surface));
	}
	.crew-picker__box:focus-within {
		border: 2px solid var(--md-sys-color-primary);
		padding: 0 13.5px;
	}
	.crew-picker__box input {
		flex: 1;
		min-width: 0;
		height: 100%;
		border: none;
		background: none;
		color: var(--md-sys-color-on-surface);
		font: inherit;
		font-size: var(--md-sys-typescale-body-large-size);
		outline: none;
	}
	.crew-picker__box input::placeholder {
		color: var(--md-sys-color-on-surface);
	}
	.crew-picker__box input:focus::placeholder {
		color: var(--md-sys-color-on-surface-variant);
	}
	.crew-picker__role {
		flex: none;
		max-width: 50%;
		overflow: hidden;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.crew-picker__list {
		position: absolute;
		top: calc(100% - 2px);
		right: 0;
		left: 0;
		z-index: 5;
		max-height: 280px;
		margin: 0;
		padding: 4px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 12px;
		background: var(--md-sys-color-surface-container);
		box-shadow: var(--md-sys-elevation-shadow-level2);
		list-style: none;
		overflow-y: auto;
	}
	.crew-picker__option {
		display: flex;
		align-items: center;
		gap: 12px;
		min-height: 48px;
		padding: 4px 10px;
		border-radius: 8px;
		cursor: pointer;
	}
	.crew-picker__option--hi {
		background: var(--md-sys-color-surface-container-highest);
	}
	.crew-picker__option[aria-selected='true'] .crew-picker__name {
		color: var(--md-sys-color-primary);
	}
	.crew-picker__text {
		display: flex;
		flex-direction: column;
		min-width: 0;
	}
	.crew-picker__name {
		font-weight: 600;
		color: var(--md-sys-color-on-surface);
	}
	.crew-picker__what {
		overflow: hidden;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.crew-picker__none {
		padding: 12px 10px;
		color: var(--md-sys-color-on-surface-variant);
	}
	@media (max-width: 600px) {
		.crew-picker__role {
			display: none;
		}
	}
</style>
