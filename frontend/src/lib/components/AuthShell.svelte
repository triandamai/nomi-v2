<script lang="ts">
	import type { Snippet } from 'svelte';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';

	// Shared frame for login/register: the crew of agent shapes on a dark panel beside the form.
	// Stacks (panel on top, compact) below 900px.

	let { title, children }: { title: string; children: Snippet } = $props();

	const CREW = ['money', 'coding', 'planning', 'personality'] as const;
</script>

<div class="auth">
	<aside class="auth__brand" aria-hidden="true">
		<div class="auth__mark">
			<AgentShape size={56} face />
			<span class="auth__word">nomi</span>
		</div>
		<p class="auth__line">A small crew of agents that remembers you.</p>
		<div class="auth__crew">
			{#each CREW as agent, i (agent)}
				<span class="auth__crew-item" style="--i: {i}"><AgentShape {agent} size={72} /></span>
			{/each}
		</div>
	</aside>

	<main class="auth__main">
		<div class="auth__card">
			<h1 class="auth__title">{title}</h1>
			{@render children()}
		</div>
	</main>
</div>

<style>
	.auth {
		min-height: 100vh;
		display: flex;
		flex-wrap: wrap;
		background: var(--md-sys-color-surface);
	}
	.auth__brand {
		flex: 1 1 480px;
		display: flex;
		flex-direction: column;
		justify-content: space-between;
		gap: 32px;
		margin: 12px;
		padding: clamp(28px, 5vw, 56px);
		border-radius: 40px 40px 40px 12px;
		background: var(--nomi-color-stage);
		color: var(--nomi-color-on-stage);
		overflow: hidden;
	}
	.auth__mark {
		display: flex;
		align-items: center;
		gap: 14px;
	}
	.auth__word {
		font-family: var(--md-ref-typeface-brand);
		font-size: 3rem;
		line-height: 1;
		font-weight: 800;
		letter-spacing: -0.045em;
	}
	.auth__line {
		margin: 0;
		max-width: 16ch;
		font-family: var(--md-ref-typeface-brand);
		font-size: clamp(2.25rem, 3.6vw, 3.75rem);
		line-height: 1;
		font-weight: 800;
		letter-spacing: -0.04em;
	}
	.auth__crew {
		display: flex;
		gap: 12px;
		flex-wrap: wrap;
	}
	.auth__crew-item {
		display: inline-flex;
		animation: auth-crew-in var(--nomi-motion-spatial-slow) both;
		animation-delay: calc(var(--i) * 80ms + 120ms);
	}
	@keyframes auth-crew-in {
		from {
			scale: 0.4;
			rotate: -40deg;
			opacity: 0;
		}
	}

	.auth__main {
		flex: 1 1 420px;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 32px 16px;
		box-sizing: border-box;
	}
	.auth__card {
		width: 100%;
		max-width: 400px;
		display: flex;
		flex-direction: column;
		gap: 16px;
	}
	.auth__title {
		margin: 0 0 8px;
		font-family: var(--md-ref-typeface-brand);
		font-size: 2.5rem;
		line-height: 1.05;
		font-weight: 800;
		letter-spacing: -0.035em;
		color: var(--md-sys-color-on-surface);
	}

	@media (max-width: 900px) {
		.auth__brand {
			flex-basis: 100%;
			gap: 16px;
		}
		.auth__line {
			font-size: 2rem;
		}
		.auth__crew {
			display: none;
		}
	}
</style>
