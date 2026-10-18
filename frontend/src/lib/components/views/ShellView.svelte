<!-- A command and what it printed, as a terminal: the prompt line, the output (scrolls when long,
     opened at its end), and how it exited. Copy takes the command. -->
<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import type { ShellView, ViewProps } from '$lib/views/types';

	let { view }: ViewProps<ShellView> = $props();

	let copied = $state(false);
	function copy() {
		navigator.clipboard?.writeText(view.command);
		copied = true;
		setTimeout(() => (copied = false), 1500);
	}

	// Long output opens scrolled to its end, where results and errors are.
	function toEnd(node: HTMLElement) {
		node.scrollTop = node.scrollHeight;
	}
</script>

<section class="shell" aria-label={m.view_shell()}>
	<div class="shell__bar">
		<span class="shell__cwd">{view.cwd ?? ''}</span>
		{#if view.exit_code !== undefined && view.exit_code !== null}
			<span class="shell__exit" class:shell__exit--failed={view.exit_code !== 0}>{m.view_exit_code({ code: String(view.exit_code) })}</span>
		{/if}
		<button type="button" class="shell__copy" onclick={copy} aria-label={copied ? m.msg_copied() : m.view_copy_command()}>
			{#if copied}
				<svg width="16" height="16" viewBox="0 0 24 24" aria-hidden="true"><polyline points="5 12.5 10 17.5 19 6.5" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" /></svg>
			{:else}
				<svg width="16" height="16" viewBox="0 0 24 24" aria-hidden="true"><rect x="9" y="9" width="11" height="11" rx="1.5" fill="none" stroke="currentColor" stroke-width="2" /><path d="M5 15V6a2 2 0 0 1 2-2h9" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" /></svg>
			{/if}
		</button>
	</div>
	<pre class="shell__screen" use:toEnd><span class="shell__prompt" aria-hidden="true">$ </span><span class="shell__command">{view.command}</span>{#if view.output}{'\n'}{view.output}{/if}</pre>
</section>

<style>
	.shell {
		min-width: 0;
		overflow: hidden;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-inverse-surface);
		color: var(--md-sys-color-inverse-on-surface);
	}
	.shell__bar {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 6px 6px 6px 14px;
		border-bottom: 1px solid color-mix(in srgb, var(--md-sys-color-inverse-on-surface) 14%, transparent);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
	}
	.shell__cwd {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		opacity: 0.7;
	}
	.shell__exit {
		padding: 2px 8px;
		border-radius: var(--md-sys-shape-corner-full);
		background: color-mix(in srgb, var(--md-sys-color-inverse-on-surface) 14%, transparent);
	}
	.shell__exit--failed {
		background: var(--md-sys-color-error);
		color: var(--md-sys-color-on-error);
	}
	.shell__copy {
		display: grid;
		place-items: center;
		width: 32px;
		height: 32px;
		border: none;
		border-radius: var(--md-sys-shape-corner-full);
		background: transparent;
		color: inherit;
		cursor: pointer;
	}
	.shell__copy:hover {
		background: color-mix(in srgb, var(--md-sys-color-inverse-on-surface) 12%, transparent);
	}
	.shell__copy:focus-visible {
		outline: 2px solid var(--md-sys-color-inverse-primary);
		outline-offset: 1px;
	}
	.shell__screen {
		max-height: 320px;
		margin: 0;
		padding: 12px 14px;
		overflow: auto;
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.8125rem;
		line-height: 1.5;
		white-space: pre-wrap;
		overflow-wrap: anywhere;
	}
	.shell__prompt {
		color: var(--md-sys-color-inverse-primary);
		user-select: none;
	}
	.shell__command {
		font-weight: 700;
	}
	@media (max-width: 600px) {
		.shell__copy {
			width: 44px;
			height: 44px;
		}
	}
</style>
