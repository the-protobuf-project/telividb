/**
 * What the models panel is doing right now.
 *
 * A class rather than component state, for two reasons. It is testable without
 * a DOM — the polling and the "which button does this row show" rules are worth
 * asserting directly. And it takes the client as a constructor argument, so a
 * test drives it with a stub and the panel is left with nothing to do but
 * render.
 */

import { isFinished } from "$lib/api";
import type { CatalogModel, Installation, TelividbClient } from "$lib/api";

/**
 * How often a running installation is polled, in milliseconds.
 *
 * The engine reports progress per chunk, so a faster poll only re-reads the
 * same number. Slower than this and a small model finishes between ticks,
 * leaving the bar to jump from nothing to done.
 */
const POLL_MS = 500;

/** The models panel's state. */
export class ModelsState {
  /** Everything on offer, in catalog order. */
  public models = $state<CatalogModel[]>([]);
  /** Installations in flight or finished, keyed by catalog id. */
  public installs = $state<Record<string, Installation>>({});
  /** What went wrong, as the engine phrased it. */
  public error = $state<string | null>(null);
  /** Whether the catalog is being read. */
  public loading = $state(false);
  /**
   * Which model the detail column is showing.
   *
   * A id rather than the object: the list is replaced wholesale on every poll,
   * so holding the object would pin a stale copy whose install progress never
   * moved again.
   */
  public selected = $state<string | null>(null);

  /** Timers for the installations being polled, so they can be stopped. */
  private timers = new Map<string, ReturnType<typeof setInterval>>();

  public constructor(
    private readonly client: TelividbClient,
    /** Called once a model becomes resident. */
    private readonly oninstalled?: () => void,
  ) {}

  /** Read the catalog. */
  public async load(): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      this.models = await this.client.listModels();
    } catch (cause) {
      this.error = String(cause);
    } finally {
      this.loading = false;
    }
  }

  /**
   * Start installing, and follow it.
   *
   * Not guarded against a second click: the engine returns the running
   * installation rather than starting a second transfer, so a double click is
   * already harmless and a guard here would only add a way to disagree with it.
   */
  public async install(id: string): Promise<void> {
    this.error = null;
    try {
      const started = await this.client.installModel(id);
      this.installs = { ...this.installs, [id]: started };
      this.follow(id, started.name);
    } catch (cause) {
      this.error = String(cause);
    }
  }

  /** Stop an installation, keeping its partial file so a retry resumes. */
  public async cancel(id: string): Promise<void> {
    const running = this.installs[id];
    if (!running) return;
    try {
      const stopped = await this.client.cancelInstallation(running.name);
      this.installs = { ...this.installs, [id]: stopped };
    } catch (cause) {
      this.error = String(cause);
    }
  }

  /** Poll one installation until it stops. */
  private follow(id: string, name: string): void {
    this.unfollow(id);
    const timer = setInterval(async () => {
      try {
        const current = await this.client.installation(name);
        this.installs = { ...this.installs, [id]: current };
        if (isFinished(current.state)) {
          this.unfollow(id);
          // A finished install changes `installed` on the row, and only the
          // engine knows whether the digest verified — so the catalog is
          // re-read rather than the row being marked locally.
          if (current.state === "succeeded") {
            await this.load();
            // The engine loads the model as the download finishes, so what the
            // window can do has changed — text search and text import are
            // refused without one, and nothing else would tell it.
            this.oninstalled?.();
          }
        }
      } catch (cause) {
        this.error = String(cause);
        this.unfollow(id);
      }
    }, POLL_MS);
    this.timers.set(id, timer);
  }

  /** Stop polling one installation. */
  private unfollow(id: string): void {
    const timer = this.timers.get(id);
    if (timer !== undefined) {
      clearInterval(timer);
      this.timers.delete(id);
    }
  }

  /**
   * Stop every timer.
   *
   * Called when the panel unmounts. Without it a closed panel keeps polling the
   * engine forever, which is invisible until the log fills with it.
   */
  public dispose(): void {
    for (const timer of this.timers.values()) clearInterval(timer);
    this.timers.clear();
  }

  /** Whether anything is currently transferring. */
  public get busy(): boolean {
    return Object.values(this.installs).some((i) => !isFinished(i.state));
  }
}
