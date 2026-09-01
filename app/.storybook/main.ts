/**
 * Storybook, scoped to the design system rather than the whole app.
 *
 * Stories live beside the components they exercise, so a component and the
 * cases that prove it move together. The alternative — a parallel `stories/`
 * tree — drifts the moment someone renames a prop.
 */

import type { StorybookConfig } from "@storybook/sveltekit";

const config: StorybookConfig = {
  stories: ["../src/lib/**/*.stories.svelte", "../src/lib/**/*.mdx"],
  addons: ["@storybook/addon-svelte-csf"],
  framework: { name: "@storybook/sveltekit", options: {} },
};

export default config;
