// Plan bubbles load their plan once; this tells them to look again when something that can change
// a plan's state happens in the open chat: a build was handed over or ended, or a new version came.

export const planRefresh = $state({ revision: 0 });

export function refreshPlans() {
	planRefresh.revision += 1;
}
