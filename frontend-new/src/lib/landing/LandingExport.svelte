<script lang="ts" module>
	export type ExportCopy = {
		title: string;
		lead: string;
		checks: string[];
		files: { chats: [string, string]; memory: [string, string]; project: [string, string] };
		panel: { title: string; ready: string; zip: string; size: string; download: string };
	};
</script>

<script lang="ts">
	import { Check } from '@lucide/svelte';
	import Icon from '$lib/ds/foundations/Icon.svelte';
	import LandingBadge from './LandingBadge.svelte';
	import LandingMascot from './LandingMascot.svelte';

	// Figma, Landing v2 → Own everything (170:2915): the words on the left, and on the right a 700×560
	// picture of an export: three files fanned out, the export panel over them and Maya breathing in
	// the corner. Two of the design's lines are changed to what Nomi does: an export holds the code
	// and its data tables (not "its change history"), and the offline line (Ollama or LM Studio on
	// your own machine) is left out, since Nomi connects Ollama Cloud only.
	type Props = { copy: ExportCopy };
	let { copy }: Props = $props();
</script>

<section class="own" aria-labelledby="landing-own-title">
	<div class="inner">
		<div class="copy">
			<h2 id="landing-own-title">{copy.title}</h2>
			<p>{copy.lead}</p>
			<ul class="checks">
				{#each copy.checks as check (check)}
					<li><span class="icon"><Icon icon={Check} size={20} /></span>{check}</li>
				{/each}
			</ul>
		</div>
		<div class="visual" aria-hidden="true">
			<div class="spot" style:left="90px" style:top="44.85px" style:width="282.1px" style:height="192.87px">
				<div class="file" style:rotate="-10deg"><span class="name">{copy.files.chats[0]}</span><span class="meta">{copy.files.chats[1]}</span></div>
			</div>
			<div class="spot" style:left="356.53px" style:top="110px" style:width="280.26px" style:height="188.83px">
				<div class="file" style:rotate="9deg"><span class="name">{copy.files.memory[0]}</span><span class="meta">{copy.files.memory[1]}</span></div>
			</div>
			<div class="file flat"><span class="name">{copy.files.project[0]}</span><span class="meta">{copy.files.project[1]}</span></div>
			<div class="panel">
				<span class="row"><span class="panel-title">{copy.panel.title}</span><LandingBadge label={copy.panel.ready} tone="green" /></span>
				<span class="zip"><span>{copy.panel.zip}</span><span class="quiet">{copy.panel.size}</span></span>
				<span class="download">{copy.panel.download}</span>
			</div>
			<div class="spot maya" style:left="560px" style:top="408.81px" style:width="177.91px" style:height="177.91px">
				<span class="breathe"><LandingMascot agent="maya" size={150} /></span>
			</div>
		</div>
	</div>
</section>

<style>
	.own {
		background: var(--lp-bg-subtle);
	}

	.inner {
		display: flex;
		align-items: center;
		gap: 64px;
		max-width: 1440px;
		margin: 0 auto;
		padding: 120px 64px;
	}

	.copy {
		display: flex;
		flex: 1 0 0;
		flex-direction: column;
		gap: 20px;
		min-width: 0;
	}

	h2 {
		max-width: 520px;
		margin: 0;
		font-size: clamp(34px, 4.4vw, 56px);
		line-height: 1.04;
		font-weight: 500;
		letter-spacing: -0.04em;
	}

	.copy p {
		max-width: 500px;
		margin: 0;
		font-size: 18px;
		line-height: 1.55;
		letter-spacing: -0.18px;
		color: var(--lp-text-2);
	}

	.checks {
		display: flex;
		flex-direction: column;
		gap: 12px;
		margin: 0;
		padding: 0;
		list-style: none;
	}

	.checks li {
		display: flex;
		align-items: flex-start;
		gap: 12px;
		max-width: 472px;
		font-size: 16px;
		line-height: 1.5;
		letter-spacing: -0.16px;
	}

	.icon {
		display: flex;
		flex-shrink: 0;
		padding-top: 2px;
	}

	.visual {
		position: relative;
		flex-shrink: 0;
		width: 700px;
		height: 560px;
		overflow: hidden;
		border-radius: var(--lp-radius-2xl);
		background: var(--lp-warning-bg);
	}

	.spot {
		position: absolute;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.file {
		display: flex;
		flex-direction: column;
		justify-content: flex-end;
		gap: 2px;
		width: 260px;
		height: 150px;
		padding: 19px;
		border: 1px solid var(--lp-border);
		border-radius: var(--lp-radius-xl);
		background: var(--lp-surface);
		font-family: var(--nm-mono);
		white-space: nowrap;
	}

	.flat {
		position: absolute;
		top: 46px;
		left: 230px;
	}

	.name {
		font-size: 18px;
		line-height: 20px;
	}

	.meta {
		font-size: 12px;
		line-height: 16px;
		color: var(--lp-text-3);
	}

	.panel {
		position: absolute;
		top: 250px;
		left: 160px;
		display: flex;
		flex-direction: column;
		gap: 16px;
		width: 380px;
		padding: 19px;
		border: 1px solid var(--lp-border);
		border-radius: var(--lp-radius-xl);
		background: var(--lp-surface);
		box-shadow: var(--lp-elevation-3);
	}

	.row {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.panel-title {
		font-size: 16px;
		line-height: 22px;
		font-weight: 600;
		letter-spacing: -0.16px;
	}

	.zip {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 16px;
		border-radius: var(--lp-radius-md);
		background: var(--lp-bg-muted);
		font-family: var(--nm-mono);
		font-size: 12px;
		line-height: 16px;
	}

	.quiet {
		color: var(--lp-text-3);
	}

	.download {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 40px;
		border-radius: var(--lp-radius-sm);
		background: var(--lp-primary);
		color: var(--lp-primary-fg);
		font-size: 14px;
		line-height: 16px;
		font-weight: 500;
		letter-spacing: 0.28px;
		text-transform: uppercase;
	}

	.maya {
		rotate: -12deg;
	}

	/* lp-squash is landing.css's: wide and short, then narrow and tall, every 0.75s. */
	.breathe {
		display: flex;
		animation: lp-squash 0.75s ease-in-out infinite alternate;
	}

	@media (max-width: 1279px) {
		.inner {
			flex-direction: column;
			align-items: flex-start;
			padding: 80px 24px;
		}

		.visual {
			align-self: center;
		}
	}

	/* Too narrow for the fanned files: the export panel alone, as the picture's point. */
	@media (max-width: 767px) {
		.inner {
			padding: 64px 16px;
		}

		.visual {
			width: 100%;
			height: auto;
			padding: 24px 16px;
			overflow: visible;
		}

		.visual > :not(.panel) {
			display: none;
		}

		.panel {
			position: static;
			width: 100%;
		}
	}
</style>
