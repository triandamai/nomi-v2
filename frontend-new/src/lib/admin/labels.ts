import { m } from '$lib/paraglide/messages';
import { AREAS, type Area, type Level, type NavGroupKey, type PageKey } from './access';

// What the console calls its pages, groups, areas and levels, in the person's language.

export const PAGE_LABELS: Record<PageKey, () => string> = {
	overview: m.admin_page_overview,
	live: m.admin_page_live,
	usage: m.admin_page_usage,
	users: m.admin_page_users,
	plans: m.admin_page_plans,
	notifications: m.admin_page_notifications,
	models: m.admin_page_models,
	providers: m.admin_page_providers,
	embedding: m.admin_page_embedding,
	tools: m.admin_page_tools,
	dynamicAgents: m.admin_page_dynamic_agents,
	support: m.admin_page_support,
	maintenance: m.admin_page_maintenance,
	integrations: m.admin_page_integrations,
	configuration: m.admin_page_configuration,
};

export const GROUP_LABELS: Record<NavGroupKey, () => string> = {
	dashboard: m.admin_group_dashboard,
	people: m.admin_group_people,
	crew: m.admin_group_crew,
	operations: m.admin_group_operations,
};

const AREA_LABELS: Record<Area, [() => string, () => string]> = {
	dashboard: [m.admin_area_dashboard, m.admin_area_dashboard_desc],
	user: [m.admin_area_user, m.admin_area_user_desc],
	plans: [m.admin_area_plans, m.admin_area_plans_desc],
	notifications: [m.admin_area_notifications, m.admin_area_notifications_desc],
	models: [m.admin_area_models, m.admin_area_models_desc],
	tools: [m.admin_area_tools, m.admin_area_tools_desc],
	agents: [m.admin_area_agents, m.admin_area_agents_desc],
	support: [m.admin_area_support, m.admin_area_support_desc],
	maintenance: [m.admin_area_maintenance, m.admin_area_maintenance_desc],
	integrations: [m.admin_area_integrations, m.admin_area_integrations_desc],
	configuration: [m.admin_area_configuration, m.admin_area_configuration_desc],
};

/** Every area, as the access editor lists them. */
export const areaRows = () => AREAS.map((key) => ({ key, label: AREA_LABELS[key][0](), description: AREA_LABELS[key][1]() }));

export const areaLabel = (area: Area) => AREA_LABELS[area][0]();

export const levelLabels = () => ({ none: m.admin_level_none(), view: m.admin_level_view(), write: m.admin_level_write(), manage: m.admin_level_manage() });

export const levelLabel = (level: Level) => levelLabels()[level];
