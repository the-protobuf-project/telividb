/**
 * The frame every story renders in.
 *
 * It loads the app's real stylesheet rather than a copy, so a token changed in
 * `app.css` is a token changed here. A design system with its own second copy of
 * the truth is not a design system.
 */

import type { Preview } from "@storybook/sveltekit";
import "../src/app.css";
import "../src/lib/dev/paint.css";
import { setPaint, type PaintMode } from "../src/lib/dev/paint";

const preview: Preview = {
  parameters: {
    // The app is dark by default, so the panel behind a component should be
    // too — a control judged against white is judged against the wrong ground.
    backgrounds: { disable: true },
    controls: { matchers: { color: /(background|colour|color)$/i } },
  },

  // Starting values live here, not on `globalTypes.defaultValue`, which recent
  // Storybook ignores — that is why the inspector read `undefined` and stayed
  // off however the toolbar was set.
  initialGlobals: { paint: "off", theme: "dark" },

  globalTypes: {
    theme: {
      description: "Theme",
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

    // The layout inspector. Hover any element to see its box model and whether
    // it sits on the 4px module — which turns "the alignment is off" from an
    // opinion into a number.
    paint: {
      description: "Layout inspector",
      toolbar: {
        title: "Paint",
        icon: "ruler",
        items: [
          { value: "off", title: "Off" },
          { value: "on", title: "Outlines" },
          { value: "grid", title: "Outlines + 8px grid" },
        ],
        dynamicTitle: true,
      },
    },
  },

  decorators: [
    (story, context) => {
      const theme = String(context.globals["theme"] ?? "dark");
      const root = document.documentElement;
      root.setAttribute("data-theme", theme);
      // `app.css` binds the `dark:` variant to a class rather than a media
      // query, so a shadcn component reading `dark:` saw no change from the
      // attribute alone.
      root.classList.toggle("dark", theme === "dark");
      document.body.style.background = "var(--ground)";
      setPaint(String(context.globals["paint"] ?? "off") as PaintMode);
      return story();
    },
  ],
};

export default preview;
