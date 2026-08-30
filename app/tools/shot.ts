/**
 * Screenshot the app and the mock, so a comparison can be made rather than
 * asserted.
 *
 * # What this can and cannot see
 *
 * A plain browser cannot reach the engine: the desktop build links it in-process
 * and talks over Tauri IPC, while `api/grpc.ts` — the browser transport — is a
 * declared stub. So `bun run shot` against `bun run dev` sees onboarding and the
 * honest "that transport is not built yet" message, and nothing past it.
 *
 * That is why components are built and reviewed in **Storybook**, which needs no
 * engine at all. This script's job for the app is the shell and the onboarding
 * flow; for everything else it shoots the stories.
 *
 * This exists because `svelte-check` and a green build prove a page compiles,
 * not that it renders — several rounds of panels were reported working while
 * content was being clipped by a grid-row bug that nobody had looked at. This
 * is the looking.
 *
 * Not a test: it asserts nothing and fails only if a page will not load. Its
 * output is images for a person (or an assistant) to judge.
 *
 * Usage: `bun run shot` with the dev server already up, or `bun run shot --serve`
 * to start one for the duration.
 */

import { chromium, type Browser, type Page } from "@playwright/test";
import { shootMock } from "./shot-mock";
import { mkdir, rm } from "node:fs/promises";
import { existsSync } from "node:fs";
import path from "node:path";
import process from "node:process";

/** Where the app is served during a shoot. */
const APP = process.env["SHOT_ORIGIN"] ?? "http://localhost:5199";

/** Desktop, and the width the design was drawn at. */
const VIEWPORT = { width: 1440, height: 900 };

/**
 * Both pages are shot dark.
 *
 * The design is dark-first — light is the opt-in — but a headless browser
 * reports `prefers-color-scheme: light` by default, so the first run rendered
 * the mock on white and made every comparison meaningless.
 */
const PAGE = { viewport: VIEWPORT, colorScheme: "dark" as const };

/** Where the images land. Gitignored — a working aid, not an artefact. */
const OUT = path.join(import.meta.dir, "..", ".shots");

/** The mock's own views, addressed by its `data-view` attribute. */
const MOCK_VIEWS = [
  "workspace",
  "data",
  "models",
  "people",
  "metrics",
  "settings",
] as const;

/**
 * Give a page long enough to hydrate and paint.
 *
 * Two and a half seconds rather than a few hundred milliseconds, and that is
 * not padding: the first run of this script produced a blank white frame and
 * was briefly taken for a rendering bug. SvelteKit had simply not hydrated yet.
 * A screenshot taken too early is worse than none, because it looks like
 * evidence.
 */
async function settle(page: Page): Promise<void> {
  await page.waitForTimeout(2500);
}

/** Shoot every dock view of the running app. */
async function shootApp(browser: Browser): Promise<void> {
  const page = await browser.newPage(PAGE);
  try {
    await page.goto(APP, { waitUntil: "domcontentloaded", timeout: 15_000 });
  } catch {
    console.log("  app: not reachable at " + APP + " — skipped");
    await page.close();
    return;
  }
  await settle(page);
  await page.screenshot({ path: path.join(OUT, "app", "00-open.png") });

  // Driven through the dock rather than by URL: the shell keeps which panel is
  // showing in component state, so there is no route to visit.
  for (const view of MOCK_VIEWS) {
    const button = page.locator(`.dock-btn[aria-label="${cap(view)}"]`);
    if ((await button.count()) === 0) continue;
    await button.first().click();
    await settle(page);
    await page.screenshot({ path: path.join(OUT, "app", `${view}.png`) });
    console.log(`  app: ${view}`);
  }
  await page.close();
}

/** The dock labels are capitalised; `data-view` values are not. */
function cap(s: string): string {
  return s.charAt(0).toUpperCase() + s.slice(1);
}


/** Shoot the Storybook stories, when one is running. */
async function shootStories(browser: Browser): Promise<void> {
  const origin = process.env["SHOT_STORYBOOK"] ?? "http://localhost:6006";
  const page = await browser.newPage(PAGE);
  try {
    await page.goto(`${origin}/iframe.html?id=design-system--all&viewMode=story`, {
      waitUntil: "domcontentloaded",
      timeout: 10_000,
    });
  } catch {
    console.log("  storybook: not running — skipped");
    await page.close();
    return;
  }
  await settle(page);
  await page.screenshot({
    path: path.join(OUT, "app", "design-system.png"),
    fullPage: true,
  });
  console.log("  storybook: design-system");
  await page.close();
}

const browser = await chromium.launch();
await rm(OUT, { recursive: true, force: true });
await mkdir(path.join(OUT, "app"), { recursive: true });
await mkdir(path.join(OUT, "mock"), { recursive: true });

await shootApp(browser);
await shootMock(browser, PAGE, OUT, settle);
await shootStories(browser);

await browser.close();
console.log(`\n  written to ${OUT}`);
