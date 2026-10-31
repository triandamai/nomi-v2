import api from 'virtual:ds-api';
import { catalog } from './catalog';

// Every design-system component, as the docs list them, with its API read from the source.
export const components = catalog(Object.keys(api));
export { api };
