import { describe, expect, it } from 'vitest';
import { relations } from './relations';

describe('relations', () => {
	const files = new Map([
		['ds/actions/Button.svelte', `<script lang="ts">\n\timport Icon from '../foundations/Icon.svelte';\n</script>`],
		['ds/foundations/Icon.svelte', `<script lang="ts"></script>`],
		['ds/chat/Thought.svelte', `<script lang="ts">\n\timport Button from '$lib/ds/actions/Button.svelte';\n\timport Step from './Step.svelte';\n</script>`],
		['ds/chat/Step.svelte', `<script lang="ts"></script>`],
		['ds/actions/ButtonFixture.svelte', `<script lang="ts">\n\timport Button from './Button.svelte';\n</script>`],
		['gallery/ActionsSection.svelte', `<script lang="ts">\n\timport Button from '$lib/ds/actions/Button.svelte';\n</script>\n<GallerySection\n\ttitle="Actions"\n\tdescription="x">`],
		['gallery/chat/ThinkingSection.svelte', `<script lang="ts">\n\timport Demo from './Demo.svelte';\n</script>\n<GallerySection title="Thinking & planning" description="x">`],
		['gallery/chat/Demo.svelte', `<script lang="ts">\n\timport Thought from '$lib/ds/chat/Thought.svelte';\n</script>`],
		['gallery/CrewColourSection.svelte', `<script lang="ts">\n\timport Button from '$lib/ds/actions/Button.svelte';\n</script>\n\n<section class="flex" aria-label="Crew colour">\n\t<p aria-label="not a section"></p>`],
		['gallery/build/CodeSection.svelte', `<script lang="ts">\n\timport Thought from '$lib/ds/chat/Thought.svelte';\n</script>\n<GallerySection title="Code, terminal & phone" description="x">`],
	]);
	const result = relations(files);

	it('finds the sheet sections that show each component, through gallery helpers too', () => {
		expect(result['actions/Button.svelte'].sections).toEqual([
			{ sheet: 'nomi', title: 'Actions', id: 'actions' },
			{ sheet: 'nomi', title: 'Crew colour', id: 'crew-colour' },
		]);
		expect(result['chat/Thought.svelte'].sections).toEqual([
			{ sheet: 'chat', title: 'Thinking & planning', id: 'thinking-planning' },
			{ sheet: 'build', title: 'Code, terminal & phone', id: 'code-terminal-phone' },
		]);
		expect(result['foundations/Icon.svelte'].sections).toEqual([]);
	});

	it('lists what each component is built from and what is built from it, leaving fixtures out', () => {
		expect(result['chat/Thought.svelte'].uses).toEqual(['actions/Button.svelte', 'chat/Step.svelte']);
		expect(result['actions/Button.svelte'].usedBy).toEqual(['chat/Thought.svelte']);
		expect(result['foundations/Icon.svelte'].usedBy).toEqual(['actions/Button.svelte']);
		expect(result['actions/ButtonFixture.svelte']).toBeUndefined();
	});
});
