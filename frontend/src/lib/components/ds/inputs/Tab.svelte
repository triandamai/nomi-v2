<script lang="ts">
	// An M3 primary tab: the open one in primary with an indicator under its label. Presentational.
	type Props = { label: string; selected?: boolean };
	let { label, selected = false }: Props = $props();
</script>

<span class="tab" class:tab--selected={selected}>{label}</span>

<style>
	.tab {
		position: relative;
		display: inline-flex;
		height: 48px;
		align-items: center;
		padding: 0 16px;
		color: var(--md-sys-color-on-surface-variant);
		font: var(--md-sys-typescale-title-small-weight) var(--md-sys-typescale-title-small-size) / var(--md-sys-typescale-title-small-line-height) var(--md-sys-typescale-title-small-font);
	}
	.tab--selected {
		color: var(--md-sys-color-primary);
	}
	.tab--selected::after {
		content: '';
		position: absolute;
		inset-inline: 16px;
		bottom: 0;
		height: 3px;
		border-radius: 3px 3px 0 0;
		background: var(--md-sys-color-primary);
	}
</style>
