/**
 * The first run, as four steps.
 *
 * Model, organization, project, space — in that order, because each needs the
 * one before it: a project is named inside an organization, and a space without
 * a model behind it can hold text nothing can search.
 *
 * A step advances only when its work *succeeded*. The previous version created
 * an organization and dropped the person into an empty app, which looked like
 * the flow had finished when three quarters of it had not run.
 */

import type { CatalogModel, TelividbClient } from "$lib/api";
import type { Protection } from "$lib/ui";
import { suggestId } from "$lib/api";

/** What the stepper shows. */
export const STEPS = ["Model", "Organization", "Project", "Space"] as const;

/** The first run's state, and the calls that move it along. */
export class Flow {
  /** Which step is showing, zero-based. */
  public step = $state(0);
  /** What went wrong, as the engine phrased it. */
  public error = $state<string | null>(null);
  /** Whether a call is in flight. */
  public busy = $state(false);

  /** Models the engine offers. */
  public models = $state<CatalogModel[]>([]);
  /** Which one to install. */
  public model = $state<string | null>(null);
  /** Whether a model is already resident, which lets step one be skipped. */
  public hasModel = $state(false);

  /** What the person typed, per step. */
  public organization = $state("");
  public project = $state("");
  public space = $state("");
  /** Fixed at creation, so chosen before the name is committed. */
  public protection = $state<Protection>("private");

  /** The organization's resource name, once created — the parent of the rest. */
  private parent = "";

  public constructor(private readonly client: TelividbClient) {}

  /** The id a name will become, shown so it can be objected to. */
  public idFor(name: string): string {
    return suggestId(name);
  }

  /**
   * Whether the step's prerequisite is genuinely unmet, so Continue is disabled.
   *
   * Only step zero qualifies. A model has to be chosen before there is anything
   * to install, and the choice is a visible list — a disabled button there
   * points at something the reader can already see.
   *
   * The name steps deliberately do *not* qualify, which is the change: they were
   * form validation wearing a disabled button, so an empty field gave a dead
   * control and no sentence saying why. They stay pressable and answer in
   * `next()`.
   */
  public get blocked(): boolean {
    if (this.busy) return true;
    return this.step === 0 && !this.hasModel && this.model === null;
  }

  /** The name the current step is collecting, or null on the model step. */
  private get typed(): string | null {
    if (this.step === 1) return this.organization;
    if (this.step === 2) return this.project;
    if (this.step === 3) return this.space;
    return null;
  }

  /** What this step is called, for a message that names it. */
  private get noun(): string {
    return ["model", "organization", "project", "space"][this.step] ?? "step";
  }

  /** Read the catalog, and notice if a model is already loaded. */
  public async load(): Promise<void> {
    await this.guard(async () => {
      this.models = await this.client.listModels();
      this.hasModel = this.models.some((m) => m.resident);
      this.model = this.models.find((m) => m.recommended)?.id ?? null;
    });
  }

  /** Do this step's work, and advance only if it succeeded. */
  public async next(): Promise<void> {
    // Validated on press rather than by refusing to be pressed, so an empty
    // field produces a sentence instead of a control that does nothing.
    const typed = this.typed;
    if (typed !== null && this.idFor(typed).length === 0) {
      this.error = typed.trim()
        ? `A ${this.noun} name needs a letter or a digit in it — that one gives an empty id.`
        : `Type a name for the ${this.noun}.`;
      return;
    }

    const ok = await this.guard(async () => {
      if (this.step === 0) {
        // Installing is a background job; the flow does not wait for hundreds
        // of megabytes before letting someone name their organization.
        if (!this.hasModel && this.model) await this.client.installModel(this.model);
      } else if (this.step === 1) {
        const made = await this.client.createOrganization(
          this.idFor(this.organization),
          this.organization.trim(),
        );
        this.parent = made.name;
      } else if (this.step === 2) {
        await this.client.createProject(
          this.parent,
          this.idFor(this.project),
          this.project.trim(),
        );
      } else {
        await this.client.createSpace(
          this.parent,
          this.idFor(this.space),
          this.space.trim(),
          this.protection,
        );
      }
    });
    if (ok && this.step < STEPS.length - 1) this.step += 1;
    else if (ok) this.done = true;
  }

  /** Whether all four steps completed. */
  public done = $state(false);

  /** Step back, which never undoes work — only the form moves. */
  public back(): void {
    if (this.step > 0) this.step -= 1;
    this.error = null;
  }

  /** Run one call, surfacing its failure rather than throwing into the void. */
  private async guard(work: () => Promise<void>): Promise<boolean> {
    this.busy = true;
    this.error = null;
    try {
      await work();
      return true;
    } catch (cause) {
      this.error = cause instanceof Error ? cause.message : String(cause);
      return false;
    } finally {
      this.busy = false;
    }
  }
}
