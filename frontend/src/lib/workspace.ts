import type { WorkspaceService } from '$lib/types';

/** How each Google service is named, explained and drawn (a 24px stroke icon's paths). */
export const WORKSPACE_SERVICES: Record<WorkspaceService, { label: string; detail: string; icon: string }> = {
	gmail: {
		label: 'Gmail',
		detail: 'Read, search and draft replies. Sends only after you approve.',
		icon: 'M3 7a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2zM3 7l9 6 9-6',
	},
	sheets: {
		label: 'Sheets',
		detail: 'Read and update your spreadsheets.',
		icon: 'M5 3h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2zM3 9h18M3 15h18M9 3v18',
	},
	docs: {
		label: 'Docs',
		detail: 'Read, write and edit your documents.',
		icon: 'M14 3H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9zM14 3v6h6M8 13h8M8 17h5',
	},
	drive: {
		label: 'Drive',
		detail: 'Find files and folders to work with.',
		icon: 'M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z',
	},
	calendar: {
		label: 'Calendar',
		detail: 'See your schedule and add events. Invites wait for your OK.',
		icon: 'M5 5h14a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7a2 2 0 0 1 2-2zM3 10h18M8 3v4M16 3v4',
	},
};

export const WORKSPACE_SERVICE_ORDER: WorkspaceService[] = ['gmail', 'sheets', 'docs', 'drive', 'calendar'];

/** What's switched on when someone connects for the first time. */
export const DEFAULT_SERVICES: WorkspaceService[] = ['gmail', 'sheets', 'docs', 'drive'];

export function isWorkspaceService(value: string): value is WorkspaceService {
	return (WORKSPACE_SERVICE_ORDER as string[]).includes(value);
}

export function serviceLabel(service: string): string {
	return isWorkspaceService(service) ? WORKSPACE_SERVICES[service].label : service;
}

/** The Connections page link that opens the connect sheet, optionally resuming a chat. */
export function connectLink(services: string[], resumeSessionId?: string | null): string {
	const params = new URLSearchParams({ connect: '1' });
	const known = services.filter(isWorkspaceService);
	if (known.length) params.set('services', known.join(','));
	if (resumeSessionId) params.set('resume', resumeSessionId);
	return `/connections?${params}`;
}
