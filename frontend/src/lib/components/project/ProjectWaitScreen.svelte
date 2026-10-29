<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import StatusScene from '$lib/components/StatusScene.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import IconSparkle from '$lib/components/icons/IconSparkle.svelte';
	import IconChevronLeft from '$lib/components/icons/IconChevronLeft.svelte';
	import IconChevronRight from '$lib/components/icons/IconChevronRight.svelte';
	import type { RuntimePhase } from '$lib/projectRuntime.svelte';

	let {
		phase,
		phaseLabel,
		runningCommand = null,
		error = null,
		handedToKoda = false,
		logText = '',
		onRestart,
	}: {
		phase: RuntimePhase;
		phaseLabel: string;
		runningCommand?: string | null;
		error?: string | null;
		handedToKoda?: boolean;
		logText?: string;
		onRestart?: () => void;
	} = $props();

	const tips = [
		{ title: () => m.project_tip_1_title(), body: () => m.project_tip_1_body() },
		{ title: () => m.project_tip_2_title(), body: () => m.project_tip_2_body() },
		{ title: () => m.project_tip_3_title(), body: () => m.project_tip_3_body() },
		{ title: () => m.project_tip_4_title(), body: () => m.project_tip_4_body() },
		{ title: () => m.project_tip_5_title(), body: () => m.project_tip_5_body() },
		{ title: () => m.project_tip_6_title(), body: () => m.project_tip_6_body() },
		{ title: () => m.project_tip_7_title(), body: () => m.project_tip_7_body() },
	];

	let activeTip = $state(0);
	let isPaused = $state(false);
	let showLog = $state(false);

	let logBox = $state<HTMLPreElement | null>(null);
	let following = $state(true);

	$effect(() => {
		void logText;
		if (showLog && logBox && following) {
			logBox.scrollTop = logBox.scrollHeight;
		}
	});

	function onLogScroll() {
		if (logBox) {
			following = logBox.scrollHeight - logBox.scrollTop - logBox.clientHeight < 24;
		}
	}

	function nextTip() {
		activeTip = (activeTip + 1) % tips.length;
	}

	function prevTip() {
		activeTip = (activeTip - 1 + tips.length) % tips.length;
	}

	function setTip(index: number) {
		activeTip = index;
	}

	$effect(() => {
		if (isPaused) return;
		const timer = setInterval(nextTip, 5000);
		return () => clearInterval(timer);
	});
</script>

<div class="wait-screen">
	<!-- Crew Animation acting out work -->
	<div class="wait-screen__scene">
		<StatusScene variant={phase === 'failed' ? 'broken' : 'maintenance'} />
	</div>

	<!-- Status Indicator & Phase Details -->
	<div class="wait-screen__status">
		<div class="wait-screen__badge" data-phase={phase}>
			{#if phase === 'failed'}
				<span class="wait-screen__dot wait-screen__dot--error"></span>
			{:else}
				<span class="wait-screen__pulse"></span>
			{/if}
			<span class="wait-screen__phase-label">{phaseLabel}</span>
		</div>

		{#if phase === 'waiting'}
			<p class="wait-screen__desc">{m.project_waiting_body()}</p>
		{:else if phase === 'failed'}
			<p class="wait-screen__desc wait-screen__desc--error">{error || m.error_500_title()}</p>
			{#if handedToKoda}
				<p class="wait-screen__desc">{m.project_check_failed_koda()}</p>
			{/if}
			{#if onRestart}
				<div class="wait-screen__action">
					<Button variant="filled" size="s" onclick={onRestart}>{m.project_restart()}</Button>
				</div>
			{/if}
		{:else if runningCommand}
			<p class="wait-screen__desc wait-screen__desc--mono">{m.project_koda_running({ command: runningCommand })}</p>
		{/if}
	</div>

	<!-- Carousel Tips while waiting -->
	<div
		class="wait-screen__carousel"
		role="region"
		aria-label={m.project_waiting_tips_label()}
		onmouseenter={() => (isPaused = true)}
		onmouseleave={() => (isPaused = false)}
		onfocusin={() => (isPaused = true)}
		onfocusout={() => (isPaused = false)}
	>
		<div class="wait-screen__tip-header">
			<div class="wait-screen__tip-badge">
				<IconSparkle size={13} />
				<span>{m.project_tip_sparkle()}</span>
			</div>
			<span class="wait-screen__tip-count">
				{m.project_tip_counter({ current: activeTip + 1, total: tips.length })}
			</span>
		</div>

		<div class="wait-screen__tip-content">
			<h4 class="wait-screen__tip-title">{tips[activeTip].title()}</h4>
			<p class="wait-screen__tip-body">{tips[activeTip].body()}</p>
		</div>

		<div class="wait-screen__tip-nav">
			<button
				type="button"
				class="wait-screen__nav-btn"
				onclick={prevTip}
				aria-label={m.project_tip_prev()}
				title={m.project_tip_prev()}
			>
				<IconChevronLeft size={16} />
			</button>

			<div class="wait-screen__dots" role="tablist">
				{#each tips as _, i}
					<button
						type="button"
						role="tab"
						aria-selected={activeTip === i}
						class="wait-screen__dot-btn"
						class:wait-screen__dot-btn--active={activeTip === i}
						onclick={() => setTip(i)}
						aria-label={m.project_tip_counter({ current: i + 1, total: tips.length })}
					></button>
				{/each}
			</div>

			<button
				type="button"
				class="wait-screen__nav-btn"
				onclick={nextTip}
				aria-label={m.project_tip_next()}
				title={m.project_tip_next()}
			>
				<IconChevronRight size={16} />
			</button>
		</div>
	</div>

	<!-- Optional Terminal Log Expand/Collapse for developers -->
	{#if logText}
		<div class="wait-screen__log-section">
			<button
				type="button"
				class="wait-screen__log-toggle"
				onclick={() => (showLog = !showLog)}
			>
				<span>{showLog ? m.project_terminal_toggle_hide() : m.project_terminal_toggle_show()}</span>
			</button>

			{#if showLog}
				<pre
					class="wait-screen__log"
					bind:this={logBox}
					onscroll={onLogScroll}
					aria-live="off"
					aria-label={m.project_log_label()}
				>{logText}</pre>
			{/if}
		</div>
	{/if}
</div>

<style>
	.wait-screen {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: flex-start;
		gap: 16px;
		height: 100%;
		padding: 24px 16px;
		box-sizing: border-box;
		overflow-y: auto;
		text-align: center;
	}

	.wait-screen__scene {
		width: 100%;
		max-width: 360px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.wait-screen__status {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		max-width: 480px;
	}

	.wait-screen__badge {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		padding: 4px 14px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
		font-size: 0.8125rem;
		font-weight: 600;
	}

	.wait-screen__badge[data-phase='failed'] {
		background: var(--md-sys-color-error-container);
		color: var(--md-sys-color-on-error-container);
	}

	.wait-screen__pulse {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--nomi-color-primary, #32d74b);
		box-shadow: 0 0 0 0 rgba(50, 215, 75, 0.7);
		animation: wait-pulse 1.8s infinite;
	}

	.wait-screen__dot--error {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--md-sys-color-error);
	}

	@keyframes wait-pulse {
		0% {
			transform: scale(0.95);
			box-shadow: 0 0 0 0 rgba(50, 215, 75, 0.7);
		}
		70% {
			transform: scale(1);
			box-shadow: 0 0 0 8px rgba(50, 215, 75, 0);
		}
		100% {
			transform: scale(0.95);
			box-shadow: 0 0 0 0 rgba(50, 215, 75, 0);
		}
	}

	.wait-screen__desc {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
		line-height: 1.45;
		max-width: 46ch;
	}

	.wait-screen__desc--error {
		color: var(--md-sys-color-error);
		font-weight: 500;
	}

	.wait-screen__desc--mono {
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.8125rem;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		max-width: 90vw;
	}

	.wait-screen__action {
		margin-top: 6px;
	}

	/* Carousel Tips Container */
	.wait-screen__carousel {
		width: min(440px, 100%);
		background: var(--md-sys-color-surface-container, rgba(255, 255, 255, 0.04));
		border: 1px solid var(--md-sys-color-outline-variant, rgba(255, 255, 255, 0.08));
		border-radius: var(--md-sys-shape-corner-large, 16px);
		padding: 16px;
		display: flex;
		flex-direction: column;
		gap: 10px;
		text-align: left;
		box-sizing: border-box;
		transition: border-color 0.2s ease;
	}

	.wait-screen__carousel:hover {
		border-color: color-mix(in srgb, var(--md-sys-color-primary) 30%, transparent);
	}

	.wait-screen__tip-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.wait-screen__tip-badge {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		font-size: 0.75rem;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--md-sys-color-primary);
	}

	.wait-screen__tip-count {
		font-size: 0.75rem;
		color: var(--md-sys-color-outline);
		font-family: var(--md-ref-typeface-mono);
	}

	.wait-screen__tip-content {
		display: flex;
		flex-direction: column;
		gap: 4px;
		min-height: 70px;
	}

	.wait-screen__tip-title {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 0.9375rem;
		font-weight: 700;
		color: var(--md-sys-color-on-surface);
	}

	.wait-screen__tip-body {
		margin: 0;
		font-size: 0.8125rem;
		line-height: 1.45;
		color: var(--md-sys-color-on-surface-variant);
	}

	.wait-screen__tip-nav {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-top: 4px;
	}

	.wait-screen__nav-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		padding: 0;
		border-radius: 50%;
		border: none;
		background: transparent;
		color: var(--md-sys-color-on-surface-variant);
		cursor: pointer;
		transition: background-color 0.15s ease, color 0.15s ease;
	}

	.wait-screen__nav-btn:hover {
		background: var(--md-sys-color-surface-container-high, rgba(255, 255, 255, 0.08));
		color: var(--md-sys-color-on-surface);
	}

	.wait-screen__dots {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.wait-screen__dot-btn {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--md-sys-color-outline-variant, rgba(255, 255, 255, 0.2));
		border: none;
		padding: 0;
		cursor: pointer;
		transition: all 0.25s cubic-bezier(0.2, 0, 0, 1);
	}

	.wait-screen__dot-btn--active {
		width: 20px;
		border-radius: 3px;
		background: var(--md-sys-color-primary);
	}

	/* Optional Terminal Log Drawer */
	.wait-screen__log-section {
		width: min(520px, 100%);
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		margin-top: 2px;
	}

	.wait-screen__log-toggle {
		background: transparent;
		border: none;
		color: var(--md-sys-color-primary);
		font-size: 0.8125rem;
		font-weight: 500;
		cursor: pointer;
		padding: 4px 8px;
		border-radius: var(--md-sys-shape-corner-small, 8px);
		transition: background-color 0.15s ease;
	}

	.wait-screen__log-toggle:hover {
		background: var(--md-sys-color-surface-container-high, rgba(255, 255, 255, 0.05));
		text-decoration: underline;
	}

	.wait-screen__log {
		width: 100%;
		min-height: 120px;
		max-height: 220px;
		margin: 0;
		padding: 10px 12px;
		overflow: auto;
		border-radius: var(--md-sys-shape-corner-medium, 12px);
		background: #0f1a14;
		color: #d7e6dc;
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
		line-height: 1.5;
		text-align: left;
		white-space: pre-wrap;
		word-break: break-word;
		box-sizing: border-box;
	}

	@media (prefers-reduced-motion: reduce) {
		.wait-screen__pulse {
			animation: none;
		}
	}
</style>
