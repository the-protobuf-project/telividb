/**
 * What the window knows about the engine right now.
 *
 * Polled rather than pushed: the engine binds its port before it loads a model,
 * so the window can open a good twenty seconds before text is possible, and
 * nothing tells it when that changes. Kept out of the shell because it is
 * sequencing with no markup — the shell was over the line with it inline.
 */

import { client } from "./index";
import type { Capabilities } from "./engine";

/** Engine state the shell renders, and the polling that keeps it current. */
export class EngineStatus {
  /** Where the engine listens, or null when the bridge is unreachable. */
  public address = $state<string | null>(null);
  /** Whether text can be turned into vectors yet. */
  public canEmbed = $state(false);
  /** What the engine reported about its compute environment. */
  public capabilities = $state<Capabilities | null>(null);
  /** The resident model's id, named in the top bar. */
  public model = $state<string | null>(null);
  /**
   * Whether any organization exists.
   *
   * Three states, not two: `null` is "not yet known", and showing the create
   * gate then would flash a form at someone who already has data.
   */
  public hasOrganization = $state<boolean | null>(null);
  /** The organization named in the breadcrumb. */
  public organization = $state<string | null>(null);

  /** Re-read what the engine can do. */
  public refresh(): void {
    client
      .capabilities()
      .then((c) => {
        this.address = c.address;
        this.canEmbed = c.has_model;
        this.capabilities = c;
      })
      .catch(() => (this.address = null));

    client
      .listModels()
      .then((m) => (this.model = m.find((x) => x.resident)?.id ?? null))
      .catch(() => (this.model = null));
  }

  /** Ask whether anything exists yet, and remember what. */
  public refreshOrganizations(): void {
    client
      .listOrganizations()
      .then((orgs) => {
        const live = orgs.filter((o) => !o.deleted);
        this.hasOrganization = live.length > 0;
        this.organization = live[0]?.displayName ?? null;
      })
      // A failure is not an empty tenancy. Leaving it unknown keeps the gate
      // shut rather than inviting a second organization to be created over one
      // the window merely could not read.
      .catch(() => (this.hasOrganization = null));
  }
}
