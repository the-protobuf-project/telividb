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

/** The settings panel's state. */
export class SettingsState {
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
    const draft = this.drafts[id];
    if (draft === undefined || !this.dirty(id)) return;

    this.saving = id;
    this.error = null;
    try {
      await this.client.storeProviderKey(id, draft.trim());
      const { [id]: _dropped, ...rest } = this.drafts;
      this.drafts = rest;
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
      const { [id]: _dropped, ...rest } = this.drafts;
      this.drafts = rest;
      this.providers = await this.client.listProviders();
    } catch (cause) {
      this.error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      this.saving = null;
    }
  }
}
