/// <reference types="vite/client" />
/** biome-ignore-all lint/suspicious/noExplicitAny: <setup file> */
/** biome-ignore-all lint/complexity/noBannedTypes: <not required> */

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<{}, {}, any>;
  export default component;
}
