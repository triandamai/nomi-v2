<script lang="ts">
	import { agentAccent, agentTint, type OutItem } from '$lib/home';

	let { items }: { items: OutItem[] } = $props();
</script>

<ul class="out">
	{#each items as item, i (i)}
		<li>
			<a
				class="out__item"
				href={item.session_id ? `/chat/${item.session_id}` : undefined}
				style="--tint: {agentTint(item.agent)}; --accent: {agentAccent(item.agent)}"
			>
				<span class="out__dot" class:out__dot--urgent={item.kind === 'needs_you'} aria-hidden="true"></span>
				<span class="out__text">
					<span class="out__title">{item.title}</span>
					{#if item.detail}<span class="out__detail">{item.detail}</span>{/if}
				</span>
			</a>
		</li>
	{/each}
</ul>

<style>
	ul {
		list-style: none;
		margin: 0;
		padding: 0;
	}
	.out {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.out__item {
		display: flex;
		gap: 12px;
		padding: 14px;
		border-radius: 20px;
		background: color-mix(in srgb, var(--tint) 16%, var(--md-sys-color-surface-container-lowest));
		color: inherit;
		text-decoration: none;
		transition: border-radius var(--nomi-motion-spatial-fast);
	}
	a.out__item:hover {
		border-radius: var(--md-sys-shape-corner-medium);
	}
	.out__dot {
		flex: none;
		width: 10px;
		height: 10px;
		margin-top: 6px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--accent);
	}
	.out__dot--urgent {
		box-shadow: 0 0 0 4px color-mix(in srgb, var(--accent) 22%, transparent);
	}
	.out__text {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.out__title {
		font-weight: 600;
		font-size: 0.9375rem;
	}
	.out__detail {
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
		overflow: hidden;
		text-overflow: ellipsis;
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
	}
</style>
