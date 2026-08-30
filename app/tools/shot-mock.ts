/**
 * Driving the mock through its own first-run flow.
 *
 * Split out because it knows things nothing else here does: which ids the mock
 * uses, that its slug is derived on keystrokes rather than on value changes, and
 * that its views sit behind an onboarding card. None of that is the app's
 * business, and the shooter was over the file-length limit carrying it.
 */

import type { Browser, Page } from "@playwright/test";
import { existsSync } from "node:fs";
import path from "node:path";
import process from "node:process";

/** The approved design, rendered from disk for a like-for-like comparison. */
const MOCK = process.env["SHOT_MOCK"] ?? "";

/** The mock's own views, addressed by its `data-view` attribute. */
const VIEWS = ["workspace", "data", "models", "people", "metrics", "settings"] as const;

/** Shoot the mock's onboarding and every view behind it. */
export async function shootMock(
  browser: Browser,
  page_options: Parameters<Browser["newPage"]>[0],
  OUT: string,
  settle: (p: Page) => Promise<void>,
): Promise<void> {
  if (!MOCK || !existsSync(MOCK)) {
    console.log("  mock: SHOT_MOCK unset or missing — skipped");
    return;
  }
  const page = await browser.newPage(page_options);
  await page.goto("file://" + MOCK, { waitUntil: "domcontentloaded" });
  await settle(page);

  // Shot first, because it is a screen in its own right and the one the app
  // currently gets most wrong.
  await page.screenshot({ path: path.join(OUT, "mock", "00-onboarding.png") });
  console.log("  mock: onboarding");

  // Clicked through rather than short-circuited. The mock's own onboarding is
  // what seeds the tree, the turns and the stats — hiding the card instead
  // produced six identical pictures of an empty shell, and `openWorkspace()` is
  // module-scoped so it cannot be called from here.
  for (let step = 0; step < 6; step++) {
    const next = page.locator("#next:visible");
    if ((await next.count()) === 0) break;
    // Wait for the step's own field before deciding anything. Querying straight
    // after the click found the previous step still on screen, so the field
    // looked absent, Continue looked disabled, and the walk stopped at step two.
    await page
      .locator("#org-name:visible, #proj-name:visible, #new-space-name:visible")
      .first()
      .waitFor({ state: "visible", timeout: 1500 })
      .catch(() => {});
    // Each step needs its own field filled before it will advance.
    for (const id of ["#org-name", "#proj-name", "#new-space-name"]) {
      const field = page.locator(`${id}:visible`);
      if ((await field.count()) > 0 && (await field.inputValue()) === "") {
        // Typed rather than `fill()`ed: the mock derives its slug on keystrokes,
        // and a value set in one go leaves Continue disabled with the field
        // looking correctly filled — which reads as a broken mock rather than a
        // missing event.
        await field.click();
        await field.pressSequentially(
          ["Acme Research", "Retrieval", "Notes"][step % 3] ?? "Notes",
          { delay: 12 },
        );
      }
    }
    if (await next.isDisabled()) break;
    await next.click();
    await page.waitForTimeout(700);
  }
  await settle(page);

  for (const view of VIEWS) {
    // The mock toggles views by class rather than by route, so this shows one
    // and hides the rest exactly as its own dock does.
    await page.evaluate((name) => {
      document.querySelectorAll("[data-view]").forEach((el) => {
        el.classList.toggle("hidden", el.getAttribute("data-view") !== name);
      });
    }, view);
    await settle(page);
    await page.screenshot({ path: path.join(OUT, "mock", `${view}.png`) });
    console.log(`  mock: ${view}`);
  }
  await page.close();
}
