/**
 * The frame every story renders in.
 *
 * It loads the app's real stylesheet rather than a copy, so a token changed in
 * `app.css` is a token changed here. A design system with its own second copy
 * of the truth is not a design system.
 */

import type { Preview } from "@storybook/sveltekit";
import "../src/app.css";

const preview: Preview = {
  parameters: {
    // The app is dark by default, so the panel behind a component should be
    // too — a control judged against white is judged against the wrong ground.
    backgrounds: { disable: true },
    controls: { matchers: { color: /(background|colour|color)$/i } },
  },

  // Dark and light are one switch, applied the way the app applies it: a
  // `data-theme` stamp on the root, never a class on the component.
  globalTypes: {
    theme: {
      description: "Theme",
      defaultValue: "dark",
      toolbar: {
        title: "Theme",
        icon: "circlehollow",
        items: [
          { value: "dark", title: "Dark" },
          { value: "light", title: "Light" },
        ],
        dynamicTitle: true,
      },
    },
  },

  decorators: [
    (story, context) => {
      document.documentElement.setAttribute("data-theme", context.globals.theme);
      document.body.style.background = "var(--ground)";
      return story();
    },
  ],
};

export default preview;
