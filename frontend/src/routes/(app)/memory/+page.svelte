<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { deserialize, enhance } from '$app/forms';
	import { invalidateAll } from '$app/navigation';
	import Chip from '$lib/components/m3/Chip.svelte';
	import MemoryEditForm from '$lib/components/MemoryEditForm.svelte';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import ButtonGroup from '$lib/components/m3/ButtonGroup.svelte';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import Dialog from '$lib/components/m3/Dialog.svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import Pagination from '$lib/components/m3/Pagination.svelte';
	import Snackbar from '$lib/components/m3/Snackbar.svelte';
	import IconSearch from '$lib/components/icons/IconSearch.svelte';
	import IconClose from '$lib/components/icons/IconClose.svelte';
	import { GRADIENT_STOPS } from '$lib/components/m3/shapes';
	import { fitPoints, kindLabel, MEMORY_KINDS, strengthLabel, strengthPips, timeAgo } from '$lib/memory';
	import { clampPage, pageCount, pageSlice } from '$lib/pagination';
	import { projectTo3D } from '$lib/pca';
	import type { MemoryItem, MemoryKind } from '$lib/types';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	type Sort = 'recent' | 'strongest' | 'used';
	const SORTS: { value: Sort; label: string }[] = [
		{ value: 'recent', label: m.mem_sort_recent() },
		{ value: 'strongest', label: m.mem_sort_strongest() },
		{ value: 'used', label: m.mem_sort_used() },
	];

	let sort = $state<Sort>('recent');
	let query = $state('');
	let selectedId = $state<string | null>(null);
	let pendingForget = $state<MemoryItem | null>(null);
	let confirmOpen = $state(false);
	let forgetting = $state(false);
	let snackbarOpen = $state(false);
	let snackbarMessage = $state('');
	function notify(message: string) {
		snackbarMessage = message;
		snackbarOpen = true;
	}
	let kindFilter = $state<MemoryKind | null>(null);
	let editing = $state<MemoryItem | null>(null);
	let editOpen = $state(false);
	function startEdit(memory: MemoryItem) {
		editing = memory;
		editOpen = true;
	}
	// Strong memories nobody has confirmed in months: ask, a few at a time.
	const toCheck = $derived(data.memories.filter((memory) => memory.needs_check).slice(0, 3));
	async function confirmMemory(memory: MemoryItem) {
		const body = new FormData();
		body.set('id', memory.id);
		const result = deserialize(await (await fetch('?/confirm', { method: 'POST', body })).text());
		if (result.type === 'success') {
			notify(m.mem_confirmed());
			await invalidateAll();
		}
	}
	let detail = $state<MemoryItem | null>(null);
	let detailOpen = $state(false);

	const PER_PAGE = 12;
	let page = $state(1);

	const memories = $derived(data.memories);

	const visible = $derived.by(() => {
		const needle = query.trim().toLowerCase();
		const filtered = memories.filter(
			(memory) => (!needle || memory.content.toLowerCase().includes(needle)) && (!kindFilter || memory.kind === kindFilter),
		);
		const sorted = [...filtered];
		if (sort === 'recent') sorted.sort((a, b) => Date.parse(b.updated_at) - Date.parse(a.updated_at));
		if (sort === 'strongest') sorted.sort((a, b) => b.weight - a.weight);
		if (sort === 'used') sorted.sort((a, b) => b.uses - a.uses || b.weight - a.weight);
		return sorted;
	});

	const pages = $derived(pageCount(visible.length, PER_PAGE));
	const shown = $derived(pageSlice(visible, clampPage(page, pages), PER_PAGE));

	// A new search or sort starts from the first page.
	$effect(() => {
		void query;
		void sort;
		void kindFilter;
		page = 1;
	});

	function goToPage(next: number) {
		page = next;
		document.querySelector('.toolbar')?.scrollIntoView({ behavior: 'smooth', block: 'start' });
	}

	function showDetail(memory: MemoryItem) {
		detail = memory;
		detailOpen = true;
	}

	/** Marks a memory's text as clamped when it runs past its eight lines. */
	function clamped(node: HTMLElement) {
		const measure = () => node.toggleAttribute('data-clamped', node.scrollHeight > node.clientHeight + 1);
		const observer = new ResizeObserver(measure);
		observer.observe(node);
		measure();
		return { destroy: () => observer.disconnect() };
	}

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
			// The memory may be on another page of the list: go there first.
			const index = visible.findIndex((m) => m.id === id);
			if (index >= 0) page = Math.floor(index / PER_PAGE) + 1;
			requestAnimationFrame(() => document.getElementById(`memory-${id}`)?.scrollIntoView({ behavior: 'smooth', block: 'nearest' }));
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
				<h1 class="md-display-small memory__title">{m.mem_title()}</h1>
				<p class="md-body-large memory__lede">
					{m.mem_lede()}
				</p>
			</div>
			<AgentShape agent="memory" size={72} working={forgetting} class="memory__mark" />
		</header>

		{#if data.loadFailed}
			<p class="memory__notice" role="alert">{m.mem_load_failed()}</p>
		{:else if memories.length === 0}
			<section class="empty" aria-labelledby="empty-heading">
				<div class="empty__art" aria-hidden="true">
					<AgentShape agent="memory" size={132} working />
					<span class="empty__orbit empty__orbit--a"><AgentShape agent="nomi" size={40} /></span>
					<span class="empty__orbit empty__orbit--b"><AgentShape agent="planning" size={30} /></span>
					<span class="empty__orbit empty__orbit--c"><AgentShape agent="money" size={24} /></span>
				</div>
				<div class="empty__text">
					<h2 id="empty-heading" class="md-headline-small empty__title">{m.mem_empty_title()}</h2>
					<p class="md-body-large empty__body">
						{m.mem_empty_body()}
					</p>
					<Button variant="gradient" size="m" href="/">{m.mem_start_chat()}</Button>
				</div>
			</section>
		{:else}
			<section class="stage" aria-labelledby="stage-heading">
				<div class="stage__summary">
					<h2 id="stage-heading" class="stage__count">
						<span class="stage__number">{memories.length}</span>
						<span class="stage__count-label">{memories.length === 1 ? m.mem_count_one() : m.mem_count_many()}</span>
					</h2>
					<dl class="stage__stats">
						{#if newest}
							<div>
								<dt class="nomi-meta">{m.mem_newest()}</dt>
								<dd>{clip(newest.content, 64)} <span class="stage__when">{timeAgo(newest.created_at)}</span></dd>
							</div>
						{/if}
						<div>
							<dt class="nomi-meta">{m.mem_recalled()}</dt>
							<dd>
								{totalUses === 0 ? m.mem_not_used_reply() : totalUses === 1 ? m.mem_times_one() : m.mem_times_many({ count: totalUses })}
							</dd>
						</div>
						{#if mostUsed && mostUsed.uses > 0}
							<div>
								<dt class="nomi-meta">{m.mem_most_used()}</dt>
								<dd>{clip(mostUsed.content, 64)}</dd>
							</div>
						{/if}
					</dl>
				</div>

				<figure class="map">
					<svg viewBox="0 0 {MAP_SIZE} {MAP_SIZE}" class="map__svg" role="group" aria-label={m.mem_map_label()}>
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
							{m.mem_map_caption()}
						{/if}
					</figcaption>
				</figure>
			</section>

			{#if toCheck.length > 0}
				<section class="check" aria-labelledby="check-heading">
					<h2 id="check-heading" class="check__title">{m.mem_check_title()}</h2>
					<p class="check__lede">{m.mem_check_lede()}</p>
					<ul class="check__list">
						{#each toCheck as memory (memory.id)}
							<li class="check__item">
								<span class="kind">{kindLabel(memory.kind)}</span>
								<span class="check__text">{memory.content}</span>
								<span class="check__actions">
									<Button variant="tonal" size="xs" onclick={() => confirmMemory(memory)}>{m.mem_check_yes()}</Button>
									<Button variant="text" size="xs" onclick={() => startEdit(memory)}>{m.mem_fix()}</Button>
									<Button variant="text" size="xs" onclick={() => askToForget(memory)}>{m.mem_forget()}</Button>
								</span>
							</li>
						{/each}
					</ul>
				</section>
			{/if}

			<div class="toolbar">
				<label class="search">
					<IconSearch />
					<span class="sr-only">{m.mem_search()}</span>
					<input type="search" bind:value={query} placeholder={m.mem_search_placeholder()} class="search__input" />
				</label>
				<ButtonGroup options={SORTS} bind:value={sort} aria-label={m.mem_sort_label()} />
			</div>
			<div class="kinds" role="group" aria-label={m.mem_kind()}>
				<Chip variant="filter" selected={kindFilter === null} onclick={() => (kindFilter = null)}>{m.common_all()}</Chip>
				{#each MEMORY_KINDS as kind (kind)}
					<Chip variant="filter" selected={kindFilter === kind} onclick={() => (kindFilter = kindFilter === kind ? null : kind)}>{kindLabel(kind)}</Chip>
				{/each}
			</div>

			{#if visible.length === 0}
				<p class="memory__notice">{m.mem_no_match({ query: query.trim() })}</p>
			{:else}
				<ul class="cards">
					{#each shown as memory (memory.id)}
						{@const pips = strengthPips(memory.weight)}
						<li
							id="memory-{memory.id}"
							class="card"
							class:card--selected={selectedId === memory.id}
						>
							<div class="card__top">
								<span class="card__meta">
									<span class="kind">{kindLabel(memory.kind)}</span>
									<span class="nomi-meta">{m.mem_learned({ when: timeAgo(memory.created_at) })}</span>
								</span>
								<span class="card__buttons">
									<IconButton aria-label={m.mem_fix_this()} title={m.mem_fix()} onclick={() => startEdit(memory)}>
										<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M12 20h9M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4Z" /></svg>
									</IconButton>
									<IconButton aria-label={m.mem_forget_this()} title={m.mem_forget()} onclick={() => askToForget(memory)}>
										<IconClose />
									</IconButton>
								</span>
							</div>
							<div class="card__body">
								<p class="card__content" use:clamped>{memory.content}</p>
								<Button variant="text" size="xs" class="card__more" onclick={() => showDetail(memory)}>{m.mem_show_more()}</Button>
							</div>
							<div class="card__foot">
								<span class="strength" title={m.mem_weight({ weight: memory.weight.toFixed(2) })}>
									<span class="strength__pips" aria-hidden="true">
										{#each [1, 2, 3, 4, 5] as pip (pip)}
											<span class="strength__pip" class:strength__pip--on={pip <= pips}></span>
										{/each}
									</span>
									<span class="strength__label">{strengthLabel(pips)}<span class="sr-only">{m.mem_strength_sr()}</span></span>
								</span>
								<span class="card__uses">
									{memory.uses === 0 ? m.mem_not_used() : memory.uses === 1 ? m.mem_used_one() : m.mem_used_many({ count: memory.uses })}
								</span>
							</div>
						</li>
					{/each}
				</ul>
				<Pagination page={clampPage(page, pages)} {pages} onselect={goToPage} label={m.mem_pages()} />
			{/if}
		{/if}

		<section class="how" aria-labelledby="how-heading">
			<h2 id="how-heading" class="md-title-large how__title">{m.mem_how()}</h2>
			<ol class="how__steps">
				<li>
					<AgentShape agent="nomi" size={40} />
					<span><strong>{m.mem_how1_title()}</strong> {m.mem_how1()}</span>
				</li>
				<li>
					<AgentShape agent="memory" size={40} />
					<span><strong>{m.mem_how2_title()}</strong> {m.mem_how2()}</span>
				</li>
				<li>
					<AgentShape agent="planning" size={40} />
					<span><strong>{m.mem_how3_title()}</strong> {m.mem_how3()}</span>
				</li>
				<li>
					<AgentShape agent="supervisor" size={40} />
					<span><strong>{m.mem_how4_title()}</strong> {m.mem_how4()}</span>
				</li>
			</ol>
			<p class="how__strength">{m.mem_strength_explained()}</p>
		</section>
	</div>
</div>

<Dialog bind:open={confirmOpen} headline={m.mem_forget_title()}>
	{#snippet children()}
		<p class="md-body-medium dialog__quote">“{pendingForget?.content}”</p>
		<p class="md-body-medium">{m.mem_forget_body()}</p>
	{/snippet}
	{#snippet actions()}
		<Button variant="text" onclick={() => (confirmOpen = false)}>{m.mem_keep()}</Button>
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
						notify(m.mem_forgotten());
					}
				};
			}}
		>
			<input type="hidden" name="id" value={pendingForget?.id ?? ''} />
			<Button variant="filled" type="submit">{m.mem_forget()}</Button>
		</form>
	{/snippet}
</Dialog>

<BottomSheet bind:open={detailOpen}>
	{#snippet children()}
		{#if detail}
			{@const pips = strengthPips(detail.weight)}
			<article class="detail" aria-labelledby="memory-detail-title">
				<span class="nomi-meta">{m.mem_learned({ when: timeAgo(detail.created_at) })}</span>
				<h2 id="memory-detail-title" class="sr-only">{m.mem_title()}</h2>
				<p class="detail__content">{detail.content}</p>
				<dl class="detail__facts">
					<div>
						<dt class="nomi-meta">{m.mem_kind()}</dt>
						<dd>{kindLabel(detail.kind)}</dd>
					</div>
					<div>
						<dt class="nomi-meta">{m.mem_strength()}</dt>
						<dd>{strengthLabel(pips)}</dd>
					</div>
					<div>
						<dt class="nomi-meta">{m.mem_recalled()}</dt>
						<dd>{detail.uses === 0 ? m.mem_not_used() : detail.uses === 1 ? m.mem_in_one() : m.mem_in_many({ count: detail.uses })}</dd>
					</div>
					<div>
						<dt class="nomi-meta">{m.mem_updated()}</dt>
						<dd>{timeAgo(detail.updated_at)}</dd>
					</div>
				</dl>
				<div class="detail__actions">
					<Button
						variant="outlined"
						onclick={() => {
							const memory = detail;
							detailOpen = false;
							if (memory) askToForget(memory);
						}}>{m.mem_forget()}</Button
					>
					<Button
						variant="tonal"
						onclick={() => {
							const memory = detail;
							detailOpen = false;
							if (memory) startEdit(memory);
						}}>{m.mem_fix()}</Button
					>
					<Button variant="filled" onclick={() => (detailOpen = false)}>{m.common_done()}</Button>
				</div>
			</article>
		{/if}
	{/snippet}
</BottomSheet>

<BottomSheet bind:open={editOpen}>
	{#snippet children()}
		{#if editing}
			<h2 class="edit-title">{m.mem_fix_title()}</h2>
			{#key editing.id}
				<MemoryEditForm
					id={editing.id}
					content={editing.content}
					kind={editing.kind}
					oncancel={() => (editOpen = false)}
					onsaved={async () => {
						editOpen = false;
						notify(m.mem_edited());
						await invalidateAll();
					}}
				/>
			{/key}
		{/if}
	{/snippet}
</BottomSheet>

<Snackbar bind:open={snackbarOpen} message={form?.error ?? (snackbarMessage || m.mem_forgotten())} />

<style>
	.kind {
		padding: 2px 8px;
		border-radius: 999px;
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
		font-size: 0.6875rem;
		font-weight: 700;
		white-space: nowrap;
	}
	.kinds {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}
	.card__meta {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		min-width: 0;
		flex-wrap: wrap;
	}
	.card__buttons {
		display: inline-flex;
		flex: none;
	}
	.check {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 20px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-tertiary-container);
		color: var(--md-sys-color-on-tertiary-container);
	}
	.check__title {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.25rem;
		font-weight: 700;
	}
	.check__lede {
		margin: 0;
		font-size: 0.875rem;
	}
	.check__list {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.check__item {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 6px 10px;
		padding: 10px 12px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
	}
	.check__text {
		flex: 1 1 200px;
		min-width: 0;
		overflow-wrap: anywhere;
	}
	.check__actions {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
	}
	.how__strength {
		margin: 12px 0 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
		max-width: 70ch;
	}
	.edit-title {
		margin: 0 0 16px;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.5rem;
		font-weight: 700;
		color: var(--md-sys-color-on-surface);
	}
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
	.card__body {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 4px;
	}
	.card__content {
		display: -webkit-box;
		-webkit-box-orient: vertical;
		-webkit-line-clamp: 8;
		line-clamp: 8;
		overflow: hidden;
		margin: 0;
		padding-right: 10px;
		font-size: 1.0625rem;
		line-height: 1.45;
		font-weight: 550;
		color: inherit;
		overflow-wrap: anywhere;
	}
	/* "Show more" only where the text was cut off. */
	.card__body :global(.card__more) {
		display: none;
		margin-left: -12px;
	}
	.card__content:global([data-clamped]) + :global(.card__more) {
		display: inline-flex;
	}
	.detail {
		display: flex;
		flex-direction: column;
		gap: 16px;
		padding: 8px 24px 24px;
	}
	.detail__content {
		margin: 0;
		font-size: 1.125rem;
		line-height: 1.55;
		font-weight: 550;
		color: var(--md-sys-color-on-surface);
		white-space: pre-wrap;
		overflow-wrap: anywhere;
	}
	.detail__facts {
		display: flex;
		flex-wrap: wrap;
		gap: 12px 28px;
		margin: 0;
	}
	.detail__facts dd {
		margin: 4px 0 0;
		color: var(--md-sys-color-on-surface);
	}
	.detail__actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
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
