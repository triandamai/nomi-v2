<script lang="ts">
	import { touchScreen } from '$lib/touch';
	import { m } from '$lib/paraglide/messages';
	import { enhance } from '$app/forms';
	import Chip from '$lib/components/m3/Chip.svelte';
	import SendButton from '$lib/components/m3/SendButton.svelte';

	// Ask from home: the text becomes a new chat's first message (the home route's newChat action).

	let {
		suggestions = true,
		error = null,
		glass = false,
		center = false,
	}: {
		/** Starter chips under the composer. */
		suggestions?: boolean;
		error?: string | null;
		/** On the glass stage: a frosted pane. */
		glass?: boolean;
		center?: boolean;
	} = $props();

	let text = $state('');
	let submitting = $state(false);
	let input: HTMLTextAreaElement | undefined = $state();

	const SUGGESTIONS = [m.home_suggest_spending(), m.home_suggest_saturday(), m.home_suggest_website(), m.home_suggest_stretch()];

	function useSuggestion(suggestion: string) {
		text = suggestion;
		input?.focus();
	}
</script>

<div class="ask" class:ask--center={center}>
	<form
		method="POST"
		action="?/newChat"
		class="composer"
		class:composer--glass={glass}
		use:enhance={({ formElement }) => {
			submitting = true;
			// On a phone, sending puts the keyboard away.
			if (touchScreen()) (formElement.querySelector('textarea') as HTMLTextAreaElement | null)?.blur();
			return async ({ update }) => {
				await update();
				submitting = false;
			};
		}}
	>
		<label for="home-ask" class="sr-only">{m.home_message_nomi()}</label>
		<textarea id="home-ask" bind:this={input} bind:value={text} name="text" rows="2" placeholder={m.home_placeholder()} class="composer__input"></textarea>
		<div class="composer__bar">
			<span class="nomi-meta">{m.home_enter_hint()}</span>
			<SendButton working={submitting} disabled={!text.trim()} label={m.home_start_chat()} />
		</div>
	</form>
	{#if error}
		<p class="md-body-medium ask__error" role="alert">{error}</p>
	{/if}

	{#if suggestions}
		<div class="ask__chips">
			{#each SUGGESTIONS as suggestion (suggestion)}
				<Chip variant="suggestion" onclick={() => useSuggestion(suggestion)}>{suggestion}</Chip>
			{/each}
		</div>
	{/if}
</div>

<style>
	.ask {
		display: flex;
		flex-direction: column;
		gap: 20px;
		width: 100%;
	}
	.ask__error {
		color: var(--md-sys-color-error);
		margin: 0;
	}
	.composer {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 16px 12px 12px 20px;
		border-radius: var(--md-sys-shape-corner-extra-large-increased);
		background: var(--md-sys-color-surface-container-lowest);
		box-shadow:
			0 1px 0 var(--md-sys-color-outline-variant),
			0 18px 40px -28px color-mix(in srgb, var(--md-sys-color-on-surface) 45%, transparent);
		text-align: start;
	}
	/* Frosted over the glass stage; the only pane on the page that blurs what's behind it. */
	.composer--glass {
		background: color-mix(in srgb, var(--md-sys-color-surface-container-lowest) 70%, transparent);
		-webkit-backdrop-filter: blur(20px) saturate(1.5);
		backdrop-filter: blur(20px) saturate(1.5);
		box-shadow:
			inset 0 0 0 1px color-mix(in srgb, var(--md-sys-color-surface-container-lowest) 55%, transparent),
			0 18px 40px -28px color-mix(in srgb, var(--md-sys-color-on-surface) 45%, transparent);
	}
	@media (prefers-reduced-transparency: reduce) {
		.composer--glass {
			background: var(--md-sys-color-surface-container-lowest);
			-webkit-backdrop-filter: none;
			backdrop-filter: none;
		}
	}
	.composer__input {
		border: none;
		resize: none;
		outline: none;
		background: transparent;
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-sys-typescale-body-large-font);
		font-size: 1.0625rem;
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
	.ask__chips {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}
	.ask--center .ask__chips {
		justify-content: center;
	}
</style>
