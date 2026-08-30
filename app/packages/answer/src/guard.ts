/**
 * Whether this content may be sent to this provider.
 *
 * **This check runs in the window, and a check in the window is advisory.** A
 * modified frontend, or script injected through a rendered passage, can reach the
 * SDK without passing here. It is written anyway because it makes the intended
 * rule explicit and stops the honest mistake; it is not what makes a vault a
 * vault.
 *
 * What would: the engine declining to *return* protected passages for a search
 * that declares a remote destination, so this window never holds the content it
 * would have to be trusted not to forward. `telividb_providers::may_answer` is
 * the server-side half, written and unwired until that lands. Until then, do not
 * describe a vault as enforced anywhere a person can read it.
 */

import type { Provider } from "./types";

/**
 * How a space is protected, as the engine reports it.
 *
 * These are the wire's own words (`PROTECTION_NONE` and so on), not a friendlier
 * set: a second vocabulary for the same four states is two things that have to be
 * kept in step, and the one that drifts is the one nothing checks.
 */
export type Protection = "none" | "private" | "vault" | "sealed";

/** Refused because the content would leave the machine. */
export class WouldLeaveMachine extends Error {
  public constructor(
    /** The space that was being read. */
    public readonly space: string,
    /** Its protection, named as the person sees it. */
    public readonly protection: Protection,
    /** The provider that was refused. */
    public readonly provider: Provider,
  ) {
    super(
      `${space} is ${protection === "sealed" ? "sealed" : "key-wrapped"}, so its ` +
        `contents are answered by a model on this machine rather than a remote ` +
        `one. ${provider.displayName} is remote — choose a local provider, or ask ` +
        `in a private space.`,
    );
    this.name = "WouldLeaveMachine";
  }
}

/**
 * Throw if a protected space would be answered by a remote model.
 *
 * Refuses rather than warns: a vault whose passages were sent to a third party
 * after someone clicked through a dialog was never a vault.
 */
export function mayAnswer(
  space: string,
  protection: Protection,
  provider: Provider,
): void {
  const protected_ = protection === "vault" || protection === "sealed";
  if (!protected_ || provider.locality === "local") return;
  throw new WouldLeaveMachine(space, protection, provider);
}
