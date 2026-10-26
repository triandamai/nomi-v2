// Words for the Deployments sheet that depend on Shipyard's vocabulary.
import { m } from '$lib/paraglide/messages';

const PHASE_TITLES: Record<string, () => string> = {
	validate: m.pj_phase_title_validate,
	build: m.pj_phase_title_build,
	configure: m.pj_phase_title_configure,
	deploy: m.pj_phase_title_deploy,
	finalize: m.pj_phase_title_finalize,
};

/** "Build", "Deploy"…; a phase Nomi doesn't know reads as Shipyard named it. */
export function phaseTitle(phase: string): string {
	return PHASE_TITLES[phase]?.() ?? phase;
}

/** "Production" for the default environment, the environment's own name otherwise. */
export function environmentName(environment: string): string {
	return environment === 'production' ? m.pj_production() : environment;
}
