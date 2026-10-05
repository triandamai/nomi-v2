<script lang="ts">
	import { enhance } from '$app/forms';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import ButtonGroup from '$lib/components/m3/ButtonGroup.svelte';
	import Dialog from '$lib/components/m3/Dialog.svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import Snackbar from '$lib/components/m3/Snackbar.svelte';
	import IconSearch from '$lib/components/icons/IconSearch.svelte';
	import IconClose from '$lib/components/icons/IconClose.svelte';
	import { GRADIENT_STOPS } from '$lib/components/m3/shapes';
	import { fitPoints, STRENGTH_LABELS, strengthPips, timeAgo } from '$lib/memory';
	import { projectTo3D } from '$lib/pca';
	import type { MemoryItem } from '$lib/types';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	type Sort = 'recent' | 'strongest' | 'used';
	const SORTS: { value: Sort; label: string }[] = [
		{ value: 'recent', label: 'Recent' },
		{ value: 'strongest', label: 'Strongest' },
		{ value: 'used', label: 'Most used' },
	];

	let sort = $state<Sort>('recent');
	let query = $state('');
	let selectedId = $state<string | null>(null);
	let pendingForget = $state<MemoryItem | null>(null);
	let confirmOpen = $state(false);
	let forgetting = $state(false);
	let snackbarOpen = $state(false);

	const memories = $derived(data.memories);

	const visible = $derived.by(() => {
		const needle = query.trim().toLowerCase();
		const filtered = needle ? memories.filter((m) => m.content.toLowerCase().includes(needle)) : memories;
		const sorted = [...filtered];
		if (sort === 'recent') sorted.sort((a, b) => Date.parse(b.updated_at) - Date.parse(a.updated_at));
		if (sort === 'strongest') sorted.sort((a, b) => b.weight - a.weight);
		if (sort === 'used') sorted.sort((a, b) => b.uses - a.uses || b.weight - a.weight);
		return sorted;
	});

	const newest = $derived(
		memories.length ? memories.reduce((a, b) => (Date.parse(a.created_at) > Date.parse(b.created_at) ? a : b)) : null,
	);
	const totalUses = $derived(memories.reduce((sum, m) => sum + m.uses, 0));
	const mostUsed = $derived(memories.length ? memories.reduce((a, b) => (b.uses > a.uses ? b : a)) : null);

	// Memory map: the embeddings' first two principal components. Close dots are memories about
	// related things. Only memories embedded at the same width can share one projection.
	const MAP_SIZE = 100;
	const mapPoints = $derived.by(() => {
		const width = memories[0]?.embedding.length ?? 0;
		const mappable = memories.filter((m) => m.embedding.length === width && width > 0);
		const projected = fitPoints(
			projectTo3D(mappable.map((m) => m.embedding)).map(({ x, y }) => ({ x, y })),
			MAP_SIZE,
			9,
		);
		return mappable.map((memory, i) => ({ memory, ...projected[i], r: 1.4 + strengthPips(memory.weight) * 0.55 }));
	});
	const matchingIds = $derived(new Set(visible.map((m) => m.id)));
	const selected = $derived(memories.find((m) => m.id === selectedId) ?? null);

	function selectMemory(id: string) {
		selectedId = selectedId === id ? null : id;
		if (selectedId) {
			document.getElementById(`memory-${id}`)?.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
		}
	}

	function askToForget(memory: MemoryItem) {
		pendingForget = memory;
		confirmOpen = true;
	}

	function clip(text: string, max: number): string {
		return text.length > max ? `${text.slice(0, max - 1).trimEnd()}…` : text;
	}

	const uid = $props.id();
</script>

<div class="memory">
	<div class="memory__inner">
		<header class="memory__head">
			<div class="memory__titles">
				<h1 class="md-display-small memory__title">Memory</h1>
				<p class="md-body-large memory__lede">
					What Nomi has learned about you while you chat. It recalls these when they're relevant, and you can make it
					forget anything.
				</p>
			</div>
			<AgentShape agent="memory" size={72} working={forgetting} class="memory__mark" />
		</header>

		{#if data.loadFailed}
			<p class="memory__notice" role="alert">Couldn't load your memories just now. Reload the page to try again.</p>
		{:else if memories.length === 0}
			<section class="empty" aria-labelledby="empty-heading">
				<div class="empty__art" aria-hidden="true">
					<AgentShape agent="memory" size={132} working />
					<span class="empty__orbit empty__orbit--a"><AgentShape agent="nomi" size={40} /></span>
					<span class="empty__orbit empty__orbit--b"><AgentShape agent="planning" size={30} /></span>
					<span class="empty__orbit empty__orbit--c"><AgentShape agent="money" size={24} /></span>
				</div>
				<div class="empty__text">
					<h2 id="empty-heading" class="md-headline-small empty__title">Nothing remembered yet</h2>
					<p class="md-body-large empty__body">
						As you talk with Nomi, it keeps lasting facts — what you like, the people in your life, your routines — so
						you don't have to repeat yourself. They'll collect here.
					</p>
					<Button variant="gradient" size="m" href="/">Start a chat</Button>
				</div>
			</section>
		{:else}
			<section class="stage" aria-labelledby="stage-heading">
				<div class="stage__summary">
					<h2 id="stage-heading" class="stage__count">
						<span class="stage__number">{memories.length}</span>
						<span class="stage__count-label">{memories.length === 1 ? 'thing' : 'things'} Nomi remembers</span>
					</h2>
					<dl class="stage__stats">
						{#if newest}
							<div>
								<dt class="nomi-meta">Newest</dt>
								<dd>{clip(newest.content, 64)} <span class="stage__when">{timeAgo(newest.created_at)}</span></dd>
							</div>
						{/if}
						<div>
							<dt class="nomi-meta">Recalled</dt>
							<dd>
								{totalUses === 0 ? 'Not used in a reply yet' : `${totalUses} ${totalUses === 1 ? 'time' : 'times'} in Nomi's replies`}
							</dd>
						</div>
						{#if mostUsed && mostUsed.uses > 0}
							<div>
								<dt class="nomi-meta">Most used</dt>
								<dd>{clip(mostUsed.content, 64)}</dd>
							</div>
						{/if}
					</dl>
				</div>

				<figure class="map">
					<svg viewBox="0 0 {MAP_SIZE} {MAP_SIZE}" class="map__svg" role="group" aria-label="Memory map: similar memories sit closer together">
						<defs>
							<radialGradient id="{uid}-dot" cx="35%" cy="30%" r="80%">
								{#each GRADIENT_STOPS.bloom as stop, i (i)}
									<stop offset="{(i / (GRADIENT_STOPS.bloom.length - 1)) * 100}%" stop-color={stop} />
								{/each}
							</radialGradient>
						</defs>
						{#each [18, 32, 46] as ring (ring)}
							<circle cx="50" cy="50" r={ring} class="map__ring" />
						{/each}
						{#each mapPoints as point (point.memory.id)}
							<circle
								cx={point.x}
								cy={point.y}
								r={point.r}
								fill="url(#{uid}-dot)"
								class="map__dot"
								class:map__dot--dim={!matchingIds.has(point.memory.id)}
								class:map__dot--selected={selectedId === point.memory.id}
								role="button"
								tabindex="0"
								aria-label={point.memory.content}
								aria-pressed={selectedId === point.memory.id}
								onclick={() => selectMemory(point.memory.id)}
								onkeydown={(event) => {
									if (event.key === 'Enter' || event.key === ' ') {
										event.preventDefault();
										selectMemory(point.memory.id);
									}
								}}
							>
								<title>{point.memory.content}</title>
							</circle>
						{/each}
					</svg>
					<figcaption class="map__caption">
						{#if selected}
							<span class="map__selected">{selected.content}</span>
						{:else}
							Similar memories sit close together. Bigger dots are stronger. Tap one to find it.
						{/if}
					</figcaption>
				</figure>
			</section>

			<div class="toolbar">
				<label class="search">
					<IconSearch />
					<span class="sr-only">Search memories</span>
					<input type="search" bind:value={query} placeholder="Search what Nomi remembers" class="search__input" />
				</label>
				<ButtonGroup options={SORTS} bind:value={sort} aria-label="Sort memories" />
			</div>

			{#if visible.length === 0}
				<p class="memory__notice">Nothing matches “{query.trim()}”.</p>
			{:else}
				<ul class="cards">
					{#each visible as memory (memory.id)}
						{@const pips = strengthPips(memory.weight)}
						<li
							id="memory-{memory.id}"
							class="card"
							class:card--selected={selectedId === memory.id}
						>
							<div class="card__top">
								<span class="nomi-meta">Learned {timeAgo(memory.created_at)}</span>
								<IconButton aria-label="Forget this memory" title="Forget" onclick={() => askToForget(memory)}>
									<IconClose />
								</IconButton>
							</div>
							<p class="card__content">{memory.content}</p>
							<div class="card__foot">
								<span class="strength" title="Weight {memory.weight.toFixed(2)}">
									<span class="strength__pips" aria-hidden="true">
										{#each [1, 2, 3, 4, 5] as pip (pip)}
											<span class="strength__pip" class:strength__pip--on={pip <= pips}></span>
										{/each}
									</span>
									<span class="strength__label">{STRENGTH_LABELS[pips - 1]}<span class="sr-only"> strength</span></span>
								</span>
								<span class="card__uses">
									{memory.uses === 0 ? 'Not used yet' : `Used in ${memory.uses} ${memory.uses === 1 ? 'reply' : 'replies'}`}
								</span>
							</div>
						</li>
					{/each}
				</ul>
			{/if}
		{/if}

		<section class="how" aria-labelledby="how-heading">
			<h2 id="how-heading" class="md-title-large how__title">How memory works</h2>
			<ol class="how__steps">
				<li>
					<AgentShape agent="nomi" size={40} />
					<span><strong>Learns as you chat.</strong> After a reply, Nomi keeps at most one lasting fact from the exchange.</span>
				</li>
				<li>
					<AgentShape agent="memory" size={40} />
					<span><strong>Recalls what's relevant.</strong> The memories closest to your message are given to Nomi before it answers.</span>
				</li>
				<li>
					<AgentShape agent="planning" size={40} />
					<span><strong>Your feedback shapes it.</strong> A thumbs-up on a reply strengthens the memories it used; a thumbs-down weakens them.</span>
				</li>
			</ol>
		</section>
	</div>
</div>

<Dialog bind:open={confirmOpen} headline="Forget this memory?">
	{#snippet children()}
		<p class="md-body-medium dialog__quote">“{pendingForget?.content}”</p>
		<p class="md-body-medium">Nomi won't recall it in future replies. This can't be undone.</p>
	{/snippet}
	{#snippet actions()}
		<Button variant="text" onclick={() => (confirmOpen = false)}>Keep it</Button>
		<form
			method="POST"
			action="?/forget"
			use:enhance={() => {
				forgetting = true;
				confirmOpen = false;
				return async ({ result, update }) => {
					await update();
					forgetting = false;
					if (result.type === 'success') {
						if (selectedId === pendingForget?.id) selectedId = null;
						snackbarOpen = true;
					}
				};
			}}
		>
			<input type="hidden" name="id" value={pendingForget?.id ?? ''} />
			<Button variant="filled" type="submit">Forget</Button>
		</form>
	{/snippet}
</Dialog>

<Snackbar bind:open={snackbarOpen} message={form?.error ?? 'Forgotten. Nomi won’t bring that up again.'} />

<style>
	.memory {
		height: 100%;
		overflow-y: auto;
		padding: 32px clamp(16px, 4vw, 56px) 56px;
		box-sizing: border-box;
	}
	.memory__inner {
		max-width: 1180px;
		margin: 0 auto;
		display: flex;
		flex-direction: column;
		gap: 28px;
	}

	.memory__head {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: 24px;
	}
	.memory__title {
		margin: 0;
		color: var(--md-sys-color-on-surface);
	}
	.memory__lede {
		margin: 8px 0 0;
		max-width: 60ch;
		color: var(--md-sys-color-on-surface-variant);
	}
	.memory__head :global(.memory__mark) {
		flex: none;
	}
	@media (max-width: 640px) {
		.memory__head :global(.memory__mark) {
			display: none;
		}
	}
	.memory__notice {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
	}

	/* Empty state */
	.empty {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(min(100%, 300px), 1fr));
		align-items: center;
		gap: 32px;
		padding: clamp(24px, 5vw, 56px);
		border-radius: var(--md-sys-shape-corner-extra-extra-large);
		background: var(--md-sys-color-surface-container-low);
	}
	.empty__art {
		position: relative;
		display: grid;
		place-items: center;
		min-height: 240px;
	}
	.empty__orbit {
		position: absolute;
		display: flex;
	}
	.empty__orbit--a {
		top: 12%;
		left: 18%;
	}
	.empty__orbit--b {
		bottom: 14%;
		right: 22%;
	}
	.empty__orbit--c {
		top: 22%;
		right: 16%;
	}
	.empty__text {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 16px;
	}
	.empty__title {
		margin: 0;
		color: var(--md-sys-color-on-surface);
	}
	.empty__body {
		margin: 0;
		max-width: 46ch;
		color: var(--md-sys-color-on-surface-variant);
	}

	/* Stage: the dark summary panel, same material as Home's crew card. */
	.stage {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(min(100%, 340px), 1fr));
		gap: 28px;
		padding: clamp(20px, 3vw, 32px);
		border-radius: 40px 40px 40px 12px;
		background: var(--nomi-color-stage);
		color: var(--nomi-color-on-stage);
	}
	.stage__summary {
		display: flex;
		flex-direction: column;
		gap: 24px;
		min-width: 0;
	}
	.stage__count {
		margin: 0;
		display: flex;
		flex-direction: column;
	}
	.stage__number {
		font-family: var(--md-ref-typeface-brand);
		font-size: clamp(3.5rem, 8vw, 5.5rem);
		line-height: 0.95;
		font-weight: 800;
		letter-spacing: -0.04em;
		background: var(--nomi-gradient-bloom);
		-webkit-background-clip: text;
		background-clip: text;
		color: transparent;
	}
	.stage__count-label {
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.5rem;
		font-weight: 700;
		letter-spacing: -0.02em;
	}
	.stage__stats {
		margin: 0;
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	.stage__stats dt {
		color: inherit;
		opacity: 0.7;
	}
	.stage__stats dd {
		margin: 2px 0 0;
		font-size: 1rem;
		line-height: 1.4;
	}
	.stage__when {
		opacity: 0.65;
		white-space: nowrap;
	}

	.map {
		margin: 0;
		display: flex;
		flex-direction: column;
		gap: 10px;
		min-width: 0;
	}
	.map__svg {
		width: 100%;
		max-height: 340px;
		aspect-ratio: 1;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: color-mix(in srgb, var(--nomi-color-on-stage) 6%, transparent);
	}
	.map__ring {
		fill: none;
		stroke: color-mix(in srgb, var(--nomi-color-on-stage) 12%, transparent);
		stroke-width: 0.25;
		stroke-dasharray: 0.8 1.6;
	}
	.map__dot {
		cursor: pointer;
		transform-box: fill-box;
		transform-origin: center;
		transition:
			opacity var(--nomi-motion-effects-fast),
			scale var(--nomi-motion-spatial-fast);
	}
	.map__dot:hover,
	.map__dot:focus-visible {
		scale: 1.35;
		outline: none;
	}
	.map__dot--dim {
		opacity: 0.2;
	}
	.map__dot--selected {
		scale: 1.5;
		stroke: var(--nomi-color-on-stage);
		stroke-width: 0.5;
	}
	.map__caption {
		font-size: 0.875rem;
		line-height: 1.4;
		opacity: 0.8;
		min-height: 2.8em;
	}
	.map__selected {
		opacity: 1;
		font-weight: 600;
	}

	/* Search + sort */
	.toolbar {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
	}
	.search {
		flex: 1 1 280px;
		max-width: 480px;
		display: flex;
		align-items: center;
		gap: 10px;
		height: 56px;
		padding: 0 20px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface-variant);
	}
	.search:focus-within {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.search__input {
		flex: 1;
		min-width: 0;
		border: none;
		outline: none;
		background: transparent;
		color: var(--md-sys-color-on-surface);
		font: inherit;
		font-size: 1rem;
	}
	.search__input::placeholder {
		color: var(--md-sys-color-on-surface-variant);
	}

	/* Memory cards */
	.cards {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(min(100%, 300px), 1fr));
		gap: 16px;
	}
	.card {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 14px 10px 18px 20px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
		box-shadow: 0 1px 0 var(--md-sys-color-outline-variant);
		transition:
			border-radius var(--nomi-motion-spatial-fast),
			background-color var(--nomi-motion-effects-fast);
		scroll-margin: 24px;
	}
	.card:hover {
		border-radius: var(--md-sys-shape-corner-large);
	}
	.card--selected {
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
	}
	.card__top {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		min-height: 40px;
	}
	.card__content {
		margin: 0;
		padding-right: 10px;
		font-size: 1.0625rem;
		line-height: 1.45;
		font-weight: 550;
		color: inherit;
		overflow-wrap: anywhere;
	}
	.card__foot {
		margin-top: auto;
		padding-right: 10px;
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		flex-wrap: wrap;
	}
	.card__uses {
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.card--selected .card__uses {
		color: inherit;
		opacity: 0.8;
	}
	.strength {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		font-size: 0.8125rem;
		font-weight: 600;
	}
	.strength__pips {
		display: inline-flex;
		gap: 3px;
	}
	.strength__pip {
		width: 14px;
		height: 6px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container-highest);
	}
	.strength__pip--on {
		background: var(--nomi-gradient-bloom);
	}

	/* How it works */
	.how {
		margin-top: 12px;
		padding-top: 28px;
		border-top: 1px solid var(--md-sys-color-outline-variant);
	}
	.how__title {
		margin: 0 0 16px;
		font-weight: 700;
		color: var(--md-sys-color-on-surface);
	}
	.how__steps {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(min(100%, 260px), 1fr));
		gap: 20px;
	}
	.how__steps li {
		display: flex;
		align-items: flex-start;
		gap: 14px;
		font-size: 0.9375rem;
		line-height: 1.5;
		color: var(--md-sys-color-on-surface-variant);
	}
	.how__steps strong {
		color: var(--md-sys-color-on-surface);
	}
	.how__steps :global(svg) {
		flex: none;
	}

	.dialog__quote {
		margin: 0 0 12px;
		font-weight: 600;
		color: var(--md-sys-color-on-surface);
	}

	@media (prefers-reduced-motion: reduce) {
		.map__dot,
		.card {
			transition: none;
		}
	}
</style>
