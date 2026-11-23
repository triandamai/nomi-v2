<script lang="ts">
	import { ArrowUp, Mic, Plus } from '@lucide/svelte';
	import Icon from '$lib/ds/foundations/Icon.svelte';

	// Figma, Nomi Builder → Prompt Composer (170:324), as the landing page draws it twice (the hero
	// 170:2829, the closing call 170:3057): 560 wide, a 1px edge (border-strong once there are words), radius 16, 16 inset and
	// 16 between the field (18 / 24) and its toolbar (attach 32 with a border; the mic 40; Send 40).
	// Empty, Send is the component's secondary (white, bordered); with something typed, primary.
	// It's a real form: everything in it goes to the page's action, which goes through sign in to a
	// new chat with the words in it. Attach and the mic need an account, so they go there too.
	type Props = {
		value?: string;
		field?: HTMLTextAreaElement;
		label: string;
		placeholder?: string;
		action: string;
		attach: string;
		voice: string;
		send: string;
		/** Lines the field starts at (it grows as it's typed in). */
		rows?: 1 | 2;
	};
	let { value = $bindable(''), field = $bindable(), label, placeholder, action, attach, voice, send, rows = 2 }: Props = $props();
	const ready = $derived(value.trim().length > 0);
</script>

<form class={['composer', ready && 'typed']} method="POST" {action}>
	<textarea
		bind:this={field}
		bind:value
		name="text"
		{rows}
		aria-label={label}
		placeholder={placeholder ?? label}
		onkeydown={(event) => {
			if (event.key === 'Enter' && !event.shiftKey && !event.isComposing) {
				event.preventDefault();
				if (ready) event.currentTarget.form?.requestSubmit();
			}
		}}
	></textarea>
	<div class="toolbar">
		<button class="attach" type="submit" aria-label={attach}><Icon icon={Plus} size={16} /></button>
		<span class="right">
			<button class="voice" type="submit" aria-label={voice}><Icon icon={Mic} size={20} /></button>
			<button class="send" type="submit" aria-label={send} disabled={!ready}><Icon icon={ArrowUp} size={20} /></button>
		</span>
	</div>
</form>

<style>
	.composer {
		display: flex;
		flex-direction: column;
		gap: 16px;
		width: min(560px, 100%);
		padding: 16px;
		border: 1px solid var(--lp-border);
		border-radius: var(--lp-radius-lg);
		background: var(--lp-surface);
		box-shadow: var(--lp-elevation-2);
		text-align: left;
	}

	/* Empty, the edge is quiet (the closing call's composer); with words in it, strong (the hero's). */
	.typed {
		border-color: var(--lp-border-strong);
	}

	.composer:focus-within {
		box-shadow:
			var(--lp-elevation-2),
			0 0 0 3px var(--lp-bg-muted);
	}

	textarea {
		width: 100%;
		padding: 0;
		border: 0;
		resize: none;
		background: transparent;
		font: inherit;
		font-size: 18px;
		line-height: 24px;
		letter-spacing: -0.18px;
		color: var(--lp-text);
		outline: none;
		field-sizing: content;
	}

	textarea::placeholder {
		color: var(--lp-text-3);
	}

	.toolbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.right {
		display: flex;
		gap: 8px;
		align-items: center;
	}

	button {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		border-radius: 999px;
		cursor: pointer;
		color: var(--lp-text);
	}

	button:focus-visible {
		outline: 2px solid var(--lp-focus);
		outline-offset: 2px;
	}

	.attach {
		width: 32px;
		height: 32px;
		border: 1px solid var(--lp-border);
		background: var(--lp-surface);
	}

	.voice {
		width: 40px;
		height: 40px;
		border: 0;
		background: transparent;
	}

	.voice:hover,
	.attach:hover {
		background: var(--lp-bg-muted);
	}

	.send {
		width: 40px;
		height: 40px;
		border: 1px solid var(--lp-primary);
		background: var(--lp-primary);
		color: var(--lp-primary-fg);
	}

	.send:disabled {
		cursor: default;
		border-color: var(--lp-border);
		background: var(--lp-surface);
		color: var(--lp-text);
	}
</style>
