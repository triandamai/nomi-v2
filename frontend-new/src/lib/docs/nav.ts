// The docs' written pages, as the sidebar groups them. Components and patterns are listed from
// their own registries (components.ts, patterns.ts); the three pen sheets sit at the end.

export type NavLink = { title: string; path: string; blurb: string };
export type NavGroup = { title: string; links: NavLink[] };

export const guides: NavGroup[] = [
	{
		title: 'Get started',
		links: [
			{ title: 'Overview', path: '/design-system/get-started', blurb: 'What the design system is, what is in it, and where to begin.' },
			{ title: 'For designers', path: '/design-system/get-started/designers', blurb: 'Working in Nomi.pen, the sheets, and handing over a design.' },
			{ title: 'For developers', path: '/design-system/get-started/developers', blurb: 'Using the components, the tokens, and proving a screen matches the pen.' },
			{ title: 'For content & marketing', path: '/design-system/get-started/content', blurb: 'How Nomi looks and sounds, the crew, and what to use outside the app.' },
			{ title: 'Contributing', path: '/design-system/get-started/contributing', blurb: 'Changing the design system: pen first, then code, then proof.' },
		],
	},
	{
		title: 'Foundations',
		links: [
			{ title: 'Principles', path: '/design-system/foundations/principles', blurb: 'An instrument, not a toy: the five rules everything follows.' },
			{ title: 'Colour', path: '/design-system/foundations/colour', blurb: 'Neutral surfaces and one accent, in dark and light.' },
			{ title: 'Crew & mascots', path: '/design-system/foundations/crew', blurb: 'The ten agents, their colours and their mascots.' },
			{ title: 'Typography', path: '/design-system/foundations/typography', blurb: 'Geist and Geist Mono, the type scale and line heights.' },
			{ title: 'Space & layout', path: '/design-system/foundations/spacing', blurb: 'The 4px grid, the shells and their widths.' },
			{ title: 'Radius & elevation', path: '/design-system/foundations/radius-elevation', blurb: 'Two radii for the app, borders over shadows.' },
			{ title: 'Iconography', path: '/design-system/foundations/iconography', blurb: 'Lucide at a 1.5 stroke, in three sizes.' },
			{ title: 'Dark & light', path: '/design-system/foundations/theming', blurb: 'Every token has both; how a theme is chosen.' },
			{ title: 'Honest states', path: '/design-system/foundations/states', blurb: 'Loading, progress, empty, errors: show what is really happening.' },
			{ title: 'Accessibility', path: '/design-system/foundations/accessibility', blurb: 'Keyboard, focus, contrast, labels and motion.' },
		],
	},
	{
		title: 'Content',
		links: [
			{ title: 'Voice & tone', path: '/design-system/content/voice-and-tone', blurb: 'How Nomi and the crew sound.' },
			{ title: 'Writing style', path: '/design-system/content/writing', blurb: 'Sentence case, short labels, buttons that say what they do.' },
			{ title: 'Vocabulary', path: '/design-system/content/vocabulary', blurb: 'The words we use for chats, builds, the crew and money.' },
			{ title: 'Numbers, time & money', path: '/design-system/content/formatting', blurb: 'Costs, durations, timestamps, IDs and the mono that shows them.' },
			{ title: 'Languages', path: '/design-system/content/languages', blurb: 'English and Bahasa Indonesia, side by side.' },
		],
	},
	{
		title: 'Resources',
		links: [
			{ title: 'Tokens', path: '/design-system/tokens', blurb: 'Every design token, its CSS variable, its class and its values.' },
			{ title: 'Pen file & tools', path: '/design-system/resources', blurb: 'Nomi.pen, the sheets, the rules for agents and the pen tools.' },
		],
	},
];
