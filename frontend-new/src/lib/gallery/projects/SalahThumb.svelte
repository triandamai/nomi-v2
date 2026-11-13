<!--
	The salah tracker's own screen, as the pen draws it in the detail panel's preview (Project /
	Detail Panel → Preview Thumb, padding [16,18], gap 4). It's the person's app, not Nomi's UI, so
	its paper colours and its fonts are its own (scoped below) and the same in both themes.
-->
<div class="thumb flex h-full flex-col gap-1 px-4.5 py-4">
	<p class="eyebrow">NEXT PRAYER</p>
	<div class="flex items-end justify-between">
		<p class="prayer">Dhuhr</p>
		<p class="countdown">00:27:39</p>
	</div>
	<div class="track"><div class="fill"></div></div>
	<p class="done">1 of 5 completed</p>
</div>

<style>
	/* The app's palette and type, from the pen's thumb. */
	.thumb {
		background: #f3efe6;
	}
	.eyebrow,
	.done {
		color: #6e6a60;
		font-family: 'Inter', ui-sans-serif, system-ui, sans-serif;
	}
	.eyebrow {
		font-size: 9px;
		line-height: 11px;
		font-weight: 500;
		letter-spacing: 1px;
	}
	.done {
		font-size: 10px;
		line-height: 12px;
	}
	.prayer {
		color: #2a2925;
		font-family: var(--font-app);
		font-size: 28px;
		line-height: 35px;
		font-weight: 500;
	}
	.countdown {
		color: #4f7556;
		font-family: var(--font-mono);
		font-size: 22px;
		line-height: 29px;
		font-weight: 500;
	}
	.track {
		height: 4px;
		border-radius: 2px;
		background: #e6e1d6;
	}
	.fill {
		width: 64px;
		height: 4px;
		border-radius: 2px;
		background: #4f7556;
	}
</style>
