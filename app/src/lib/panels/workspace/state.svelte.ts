/**
 * The tenancy tree: organizations, and what each one holds.
 *
 * Projects and spaces are loaded per organization rather than all at once,
 * because both are listed under an organization parent — there is no call that
 * returns every project on the engine, and building one by fanning out would be
 * inventing a query the server does not offer.
 */

import type { Protection } from "@telividb/answer";
import type { Organization, Project, Space, TelividbClient } from "$lib/api";

/** The workspace panel's state. */
export class WorkspaceState {
  /** Every organization, soft-deleted ones included. */
  public organizations = $state<Organization[]>([]);
  /** Which one is open. Null before the first load, or after all are gone. */
  public selected = $state<Organization | null>(null);
  /** Projects under the open organization. */
  public projects = $state<Project[]>([]);
  /** Spaces under the open organization. */
  public spaces = $state<Space[]>([]);
  /** What went wrong, as the engine phrased it. */
  public error = $state<string | null>(null);
  /** Whether a call is in flight, so buttons can settle. */
  public busy = $state(false);
  /** Which kind of thing the create form is for, or null when it is closed. */
  public creating = $state<"organization" | "project" | "space" | null>(null);

  public constructor(private readonly client: TelividbClient) {}

  /** Open the create form for one kind of resource. */
  public startCreating(kind: "organization" | "project" | "space"): void {
    this.creating = kind;
  }

  /** Read the organizations, and open the first live one. */
  public async load(): Promise<void> {
    await this.guard(async () => {
      this.organizations = await this.client.listOrganizations();
      const keep =
        this.organizations.find((o) => o.name === this.selected?.name) ??
        this.organizations.find((o) => !o.deleted) ??
        this.organizations[0] ??
        null;
      await this.open(keep);
    });
  }

  /**
   * Open one organization and read what it holds.
   *
   * A soft-deleted organization still lists: seeing that it is there and
   * restorable is the point of a soft delete, and hiding it would make Undelete
   * unreachable from the only screen that offers it.
   */
  public async open(organization: Organization | null): Promise<void> {
    this.selected = organization;
    // Cleared before the fetch, not after it. Leaving the previous
    // organization's projects on screen while the next one loads shows one
    // organization's name above another's contents, and a failed load would
    // leave them there permanently.
    this.projects = [];
    this.spaces = [];
    if (!organization) return;

    // Through `guard` because this is also called directly from the rail, where
    // nothing else catches: an unguarded rejection there left the panel empty
    // with no error and an unhandled promise in the console.
    await this.guard(async () => {
      const [projects, spaces] = await Promise.all([
        this.client.listProjects(organization.name),
        this.client.listSpaces(organization.name),
      ]);
      this.projects = projects;
      this.spaces = spaces;
    });
  }

  /** Create an organization and open it. */
  public async createOrganization(id: string, displayName: string): Promise<boolean> {
    return await this.guard(async () => {
      await this.client.createOrganization(id, displayName);
      this.organizations = await this.client.listOrganizations();
      const made = this.organizations.find((o) => o.name.endsWith(`/${id}`));
      await this.open(made ?? this.selected);
    });
  }

  /** Create a project under the open organization. */
  public async createProject(id: string, displayName: string): Promise<boolean> {
    const parent = this.selected;
    if (!parent) return false;
    return await this.guard(async () => {
      await this.client.createProject(parent.name, id, displayName);
      this.projects = await this.client.listProjects(parent.name);
    });
  }

  /**
   * Create a space at the currently chosen protection.
   *
   * An arrow property rather than a method so it can be handed to a child as a
   * callback without losing `this` — the create row takes `(id, name)` and knows
   * nothing about protection, which is chosen beside it.
   */
  public createSpaceWith = (id: string, displayName: string): Promise<boolean> =>
    this.createSpace(id, displayName, this.protection);

  /** Protection for the next space. Fixed at creation, so chosen before it. */
  public protection = $state<Protection>("private");

  /** Create a space under the open organization, with its protection fixed. */
  public async createSpace(
    id: string,
    displayName: string,
    protection: Protection,
  ): Promise<boolean> {
    const parent = this.selected;
    if (!parent) return false;
    return await this.guard(async () => {
      await this.client.createSpace(parent.name, id, displayName, protection);
      this.spaces = await this.client.listSpaces(parent.name);
    });
  }

  /** Soft-delete or restore an organization, whichever it currently needs. */
  public async toggleDeleted(organization: Organization): Promise<void> {
    await this.guard(async () => {
      if (organization.deleted) {
        await this.client.undeleteOrganization(organization.name);
      } else {
        await this.client.deleteOrganization(organization.name);
      }
      this.organizations = await this.client.listOrganizations();
      const same = this.organizations.find((o) => o.name === organization.name);
      await this.open(same ?? null);
    });
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
