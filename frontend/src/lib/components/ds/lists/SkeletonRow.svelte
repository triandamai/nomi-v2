<script lang="ts">
	// Stands in while something loads, and says what ("loading 4 projects…"). Still, not shimmering:
	// honest states, no fake progress.
	let { label }: { label: string } = $props();
</script>

<div role="status" aria-busy="true" class="skeleton">
	<div class="shape" aria-hidden="true">
		<span class="tile"></span>
		<span class="lines">
			<span class="line line--short"></span>
			<span class="line"></span>
		</span>
	</div>
	<span class="label">{label}</span>
</div>

<style>
	.skeleton {
		display: flex;
		width: 100%;
		flex-direction: column;
		gap: 12px;
		padding: 16px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-lowest);
	}
	.shape {
		display: flex;
		align-items: center;
		gap: 16px;
	}
	.tile {
		width: 40px;
		height: 40px;
		flex-shrink: 0;
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-surface-container-high);
	}
	.lines {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 8px;
	}
	.line {
		height: 10px;
		width: 100%;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container);
	}
	.line--short {
		width: 140px;
		background: var(--md-sys-color-surface-container-high);
	}
	.label {
		color: var(--md-sys-color-on-surface-variant);
		font-family: var(--md-ref-typeface-mono);
		font-size: var(--md-sys-typescale-label-small-size);
	}
</style>
