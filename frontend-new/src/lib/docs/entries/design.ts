import type { ComponentDoc } from '../types';

// Design Studio components: Sora's interactive design canvas, style-tile exploration,
// OpenPencil scene graph vector inspection, and direct click-to-pin feedback.

export default [
	{
		file: 'design/DesignCanvas.svelte',
		summary:
			"The Design Studio's spatial workspace: 2D pan-and-zoom canvas showing style-tile directions, design system artboards, flow screens, direct click-to-pin feedback, and approval handoff.",
		use: [
			'In the chat design studio pane or full design preview page.',
			'To explore visual directions and give pin-point element feedback before code generation.'
		],
		avoid: ['In the live code editor: use projects/ProjectWorkspace.svelte.'],
		dos: [
			'Keep zoom controls, device switchers, and stage filters accessible at all times.',
			'Allow seamless switching between native OpenPencil vector rendering and responsive HTML artboards.'
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
			'Inside DesignCanvas when OpenPencil vector mode is enabled.',
			'For inspecting design tokens, typography styles, and element bounding boxes directly on the canvas.'
		],
		avoid: ['When testing active HTML interactions and browser forms: switch to responsive HTML frame mode.'],
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
