<script lang="ts">
	import {
		ArrowUp,
		ArrowUpRight,
		Bell,
		Check,
		CircleAlert,
		Database,
		Download,
		Folder,
		GitBranch,
		GitFork,
		Globe,
		House,
		Inbox,
		MessageSquare,
		Mic,
		Paperclip,
		Plus,
		Rocket,
		RotateCcw,
		Search,
		Settings,
		Users,
		Wallet,
		X,
	} from '@lucide/svelte';
	import Icon from '$lib/ds/foundations/Icon.svelte';

	const spacing: [step: string, use: string][] = [
		['2xs', '2px · nav item stack'],
		['xs', '4px · label to value'],
		['sm', '6px · icon to label'],
		['md', '8px · chip groups'],
		['lg', '12px · row content'],
		['xl', '16px · panel padding'],
		['2xl', '20px · panel side pad'],
		['3xl', '24px · section gap'],
		['4xl', '32px · page padding'],
		['5xl', '48px · page header gap'],
	];
	// The pen's icon grid, row by row (Block / Iconography).
	const icons = [
		[
			['house', House],
			['message-square', MessageSquare],
			['folder', Folder],
			['wallet', Wallet],
			['bell', Bell],
			['git-fork', GitFork],
			['inbox', Inbox],
			['search', Search],
		],
		[
			['plus', Plus],
			['arrow-up', ArrowUp],
			['mic', Mic],
			['paperclip', Paperclip],
			['rotate-ccw', RotateCcw],
			['download', Download],
			['git-branch', GitBranch],
			['globe', Globe],
		],
		[
			['database', Database],
			['rocket', Rocket],
			['circle-alert', CircleAlert],
			['check', Check],
			['x', X],
			['settings', Settings],
			['users', Users],
			['arrow-up-right', ArrowUpRight],
		],
	] as const;
	const iconSizes = [16, 14, 12] as const;
</script>

{#snippet head(title: string, description: string)}
	<p class="text-body font-medium text-text">{title}</p>
	<p class="text-caption text-text-3">{description}</p>
{/snippet}

<!-- As the pen draws it (Section / Space Radius Icons): no section head, three bordered blocks in a row. -->
<section id="space-radius-icons" class="flex flex-col gap-6 lg:flex-row lg:items-start" aria-label="Space, radius & icons">
	<div class="flex min-w-0 flex-1 flex-col gap-4 rounded-md p-6 outline -outline-offset-1 outline-border">
		{@render head('Spacing', '4px base with 2 / 6 for dense UI. Gaps and padding only from this scale.')}
		{#each spacing as [step, use] (step)}
			<div class="flex flex-wrap items-center gap-3 lg:flex-nowrap">
				<span class="w-20 shrink-0 font-mono text-label text-text">space-{step}</span>
				<!-- Three times the step, as drawn (6–144px). The pen rounds the bar by 2px; 2xs (3px) is the nearest radius. -->
				<span data-space-bar class="h-2 shrink-0 rounded-2xs bg-accent" style="width: calc(var(--nm-space-{step}) * 3)"></span>
				<span class="min-w-0 font-mono text-label text-text-3">{use}</span>
			</div>
		{/each}
	</div>

	<div class="flex flex-col gap-4 rounded-md p-6 outline -outline-offset-1 outline-border lg:w-90 lg:shrink-0">
		{@render head('Radius & depth', 'Two radii. Borders define edges; one shadow level, only for floating layers.')}
		<div class="flex gap-4">
			<div class="flex flex-1 flex-col gap-2">
				<div class="h-14 rounded-sm border border-border-strong bg-surface"></div>
				<span class="font-mono text-label text-text">radius-sm · 6px</span>
				<span class="text-caption text-text-3">buttons, chips, rows, inputs</span>
			</div>
			<div class="flex flex-1 flex-col gap-2">
				<div class="h-14 rounded-md border border-border-strong bg-surface"></div>
				<span class="font-mono text-label text-text">radius-md · 10px</span>
				<span class="text-caption text-text-3">panels, composer, callouts</span>
			</div>
		</div>
		<div class="flex gap-4">
			<div class="flex flex-1 flex-col gap-2">
				<div class="h-14 rounded-md border border-border bg-surface"></div>
				<span class="font-mono text-label text-text">Flat + border</span>
			</div>
			<div class="flex flex-1 flex-col gap-2">
				<div class="h-14 rounded-md border border-border-strong bg-surface shadow-float"></div>
				<span class="font-mono text-label text-text">Floating</span>
			</div>
		</div>
	</div>

	<div class="flex flex-col gap-4 rounded-md p-6 outline -outline-offset-1 outline-border lg:w-90 lg:shrink-0">
		{@render head('Iconography', 'Lucide only, 1.5 stroke. 16px in nav and buttons, 14px inline with text, 12px in chips.')}
		<div class="flex flex-col gap-3">
			{#each icons as row, index (index)}
				<div class="flex flex-wrap justify-between gap-y-3">
					{#each row as [name, glyph] (name)}
						<span data-icon-tile={name} class="flex size-8 items-center justify-center rounded-sm bg-surface text-text-2">
							<Icon icon={glyph} size={16} />
						</span>
					{/each}
				</div>
			{/each}
		</div>
		<div class="flex items-end gap-4">
			{#each iconSizes as size (size)}
				<div class="flex flex-col items-center gap-1.5 text-text">
					<Icon icon={Folder} {size} />
					<span class="font-mono text-label text-text-3">{size}px</span>
				</div>
			{/each}
		</div>
	</div>
</section>
