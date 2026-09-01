/**
 * The half of the Tauri client that configures rather than queries: providers,
 * their credentials, and the tenancy tree.
 *
 * A base class rather than a second object because `TauriClient` must satisfy
 * one port, and these methods hold no state — splitting them off is about the
 * 200-line rule and about grouping settings-shaped calls together, not about a
 * boundary that means anything at runtime.
 */

import { invoke } from "@tauri-apps/api/core";
import type { Protection, Provider } from "@telividb/answer";
import type { Organization, Project, Space } from "./tenancy";
import { Command } from "./tauri-commands";

/** Provider credentials and the tenancy tree, over Tauri IPC. */
export abstract class TauriSettings {
  /** Every model host this build knows, and whether each is ready to use. */
  public async listProviders(): Promise<Provider[]> {
    return await invoke<Provider[]>(Command.ListProviders);
  }

  /** Store a provider's credential in the OS keychain. */
  public async storeProviderKey(id: string, credential: string): Promise<void> {
    await invoke<void>(Command.StoreProviderKey, { id, credential });
  }

  /** Remove a provider's credential from the OS keychain. */
  public async forgetProviderKey(id: string): Promise<void> {
    await invoke<void>(Command.ForgetProviderKey, { id });
  }

  /**
   * Read a credential for the duration of one call.
   *
   * Not cached here on purpose: a value held in a field lives as long as the
   * window, while one fetched per call is reachable only while a request is in
   * flight. The keychain read is local and cheap.
   */
  public async providerCredential(id: string): Promise<string> {
    return await invoke<string>(Command.ProviderCredential, { id });
  }

  /** Every organization this engine holds, soft-deleted ones included. */
  public async listOrganizations(): Promise<Organization[]> {
    return await invoke<Organization[]>(Command.ListOrganizations);
  }

  /** Create an organization. */
  public async createOrganization(
    id: string,
    displayName: string,
  ): Promise<Organization> {
    return await invoke<Organization>(Command.CreateOrganization, {
      id,
      displayName,
    });
  }

  /** Soft-delete an organization. */
  public async deleteOrganization(name: string): Promise<Organization> {
    return await invoke<Organization>(Command.DeleteOrganization, { name });
  }

  /** Restore a soft-deleted organization. */
  public async undeleteOrganization(name: string): Promise<Organization> {
    return await invoke<Organization>(Command.UndeleteOrganization, { name });
  }

  /** Every project under one organization. */
  public async listProjects(parent: string): Promise<Project[]> {
    return await invoke<Project[]>(Command.ListProjects, { parent });
  }

  /** Create a project under one organization. */
  public async createProject(
    parent: string,
    id: string,
    displayName: string,
  ): Promise<Project> {
    return await invoke<Project>(Command.CreateProject, {
      parent,
      id,
      displayName,
    });
  }

  /** Every space under one organization. */
  public async listSpaces(parent: string): Promise<Space[]> {
    return await invoke<Space[]>(Command.ListSpaces, { parent });
  }

  /**
   * Create a space with its protection declared.
   *
   * `protectionKind` rather than `protection`: Tauri matches arguments by name,
   * and the Rust side names it that way to avoid colliding with the enum.
   */
  public async createSpace(
    parent: string,
    id: string,
    displayName: string,
    protection: Protection,
  ): Promise<Space> {
    return await invoke<Space>(Command.CreateSpace, {
      parent,
      id,
      displayName,
      protectionKind: protection,
    });
  }
}
