import type { ComponentDoc } from '../types';

// Design Studio components: Sora's interactive design canvas, style-tile exploration,
// OpenPencil scene graph vector inspection, and direct click-to-pin feedback.

export default [
	{
		file: 'design/DesignCanvas.svelte',
		summary:
			"The Design Studio's canvas: Sora's .pen design drawn as vector frames (older HTML versions as framed pages), with Move, Select and Comment tools, feedback pins, and approval.",
		use: [
			'In the chat design studio pane or full design preview page.',
			'To explore visual directions and give pin-point element feedback before code generation.'
		],
		avoid: ['In the live code editor: use projects/ProjectWorkspace.svelte.'],
		dos: [
			'Keep the floating tool bar (Move H, Select V, Comment C, zoom, Fit, Layers) reachable at all times.',
			'Draw a version from its own .pen files; frame only HTML pages.'
		],
		donts: ['Block navigation or canvas panning when clicking outside interactive targets.'],
		accessibility: [
			'All viewport actions (zoom in/out/fit, device selection, approvals) are keyboard-focusable and labeled.',
			'Supports standard trackpad pinch-zoom and mouse wheel navigation.'
		],
		related: ['design/OpenPencilViewer.svelte', 'projects/ProjectWorkspace.svelte']
	},
	{
		file: 'design/OpenPencilViewer.svelte',
		summary:
			'Native 60fps vector design canvas powered by the OpenPencil scene-graph engine with variable token inspection, design linting, and SVG export.',
		use: [
			'Inside DesignCanvas for every .pen version and for the proposed directions.',
			'For inspecting design tokens, typography styles, and element bounding boxes directly on the canvas.'
		],
		avoid: ['For an older HTML-only version: DesignCanvas frames its pages instead.'],
		dos: [
			'Render tokens and linter issues in the collapsible inspector sheet.',
			'Support one-click SVG export for design system artboards.'
		],
		donts: ['Recalculate layout tree on simple cursor hovers: use cached bounding boxes.'],
		accessibility: [
			'Inspector tabs and export buttons are fully keyboard-navigable and accessible with clear labels.',
			'Canvas has an aria-label and supports smooth pan-and-drag navigation.'
		],
		related: ['design/DesignCanvas.svelte']
	}
] satisfies ComponentDoc[];
