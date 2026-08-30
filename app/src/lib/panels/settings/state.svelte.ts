/**
 * What the settings panel knows.
 *
 * Two independent things: what the engine reports about itself, which is read
 * once, and the provider credentials, which are written. They share a class
 * because they share a screen, not because they are related.
 */

import type { Provider } from "@telividb/answer";
import type { Capabilities, TelividbClient } from "$lib/api";

/** What a stored credential is shown as. Never the value. */
export const MASK = "••••••••••••";

/** One section of the panel, as the left column lists them. */
export interface Section {
  /** Stable key, and the value `section` takes. */
  readonly id: string;
  /** What the left column shows. */
  readonly label: string;
  /** One line under it, so the list says what each section is for. */
  readonly summary: string;
}

/**
 * The sections, in the order they are listed.
 *
 * Ordered by how often they are opened rather than by importance: the engine's
 * own state is the thing a person checks, and About is the thing they read once.
 */
export const SECTIONS: readonly Section[] = [
  { id: "engine", label: "Engine", summary: "Backend, data directory, address" },
  { id: "answering", label: "Answering", summary: "Providers and their keys" },
  { id: "privacy", label: "Privacy", summary: "What leaves this machine" },
  { id: "about", label: "About", summary: "Version, licence, author" },
];

/** The settings panel's state. */
export class SettingsState {
  /** Which section the right-hand column is showing. */
  public section = $state("engine");
  /** What the engine reports about itself. Null until asked. */
  public capabilities = $state<Capabilities | null>(null);
  /** Every provider, with whether each is ready to use. */
  public providers = $state<Provider[]>([]);
  /** What is typed but not yet saved, by provider id. */
  public drafts = $state<Record<string, string>>({});
  /** Which provider is being written, if any. */
  public saving = $state<string | null>(null);
  /** What went wrong, as the keychain or the engine phrased it. */
  public error = $state<string | null>(null);

  public constructor(private readonly client: TelividbClient) {}

  /** Read everything this panel shows. */
  public async load(): Promise<void> {
    this.error = null;
    try {
      const [capabilities, providers] = await Promise.all([
        this.client.capabilities(),
        this.client.listProviders(),
      ]);
      this.capabilities = capabilities;
      this.providers = providers;
    } catch (cause) {
      this.error = cause instanceof Error ? cause.message : String(cause);
    }
  }

  /** What to show in a provider's field: the mask if stored, the draft if typed. */
  public shown(provider: Provider): string {
    const draft = this.drafts[provider.id];
    if (draft !== undefined) return draft;
    return provider.configured && provider.locality === "remote" ? MASK : "";
  }

  /** Whether a draft is worth writing. */
  public dirty(id: string): boolean {
    const draft = this.drafts[id];
    return draft !== undefined && draft.trim() !== "" && draft !== MASK;
  }

  /**
   * Write one credential to the OS keychain.
   *
   * The draft is dropped afterwards rather than kept: leaving the typed value in
   * a field means it survives in memory for as long as the panel is open, and
   * the mask is the honest thing to show once it is stored.
   */
  public async save(id: string): Promise<void> {
    if (!this.dirty(id)) return;
    const draft = this.drafts[id] ?? "";

    this.saving = id;
    this.error = null;
    try {
      await this.client.storeProviderKey(id, draft.trim());
      this.forget_draft(id);
      this.providers = await this.client.listProviders();
    } catch (cause) {
      this.error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      this.saving = null;
    }
  }

  /** Forget one credential. */
  public async forget(id: string): Promise<void> {
    this.saving = id;
    this.error = null;
    try {
      await this.client.forgetProviderKey(id);
      this.forget_draft(id);
      this.providers = await this.client.listProviders();
    } catch (cause) {
      this.error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      this.saving = null;
    }
  }

  /** Drop a typed value, so it does not outlive the call that used it. */
  private forget_draft(id: string): void {
    const { [id]: _dropped, ...rest } = this.drafts;
    this.drafts = rest;
  }
}
