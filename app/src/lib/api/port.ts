/**
 * What the window can ask of an engine.
 *
 * A port, not a class. The desktop app satisfies it over Tauri IPC; a browser
 * would satisfy it over gRPC-web against a remote daemon. Panels are written
 * against this and mention neither, which is what lets one codebase reach the
 * same behaviour both ways.
 */

import type { CollectionSummary } from "./collection";
import type { SearchRequest, SearchResponse } from "./search";
import type { CreateCollectionRequest, Preset } from "./preset";
import type { ImportRequest, ImportResponse, PointRow } from "./import";
import type { Capabilities } from "./engine";
import type { Protection, Provider } from "@telividb/answer";
import type { Organization, Project, Space } from "./tenancy";

/** An engine this window can reach. */
import type { CatalogModel, Installation } from "./model";

export interface TelividbClient {
  /** Collections this engine holds. */
  listCollections(): Promise<CollectionSummary[]>;
  /** Search one collection. */
  search(request: SearchRequest): Promise<SearchResponse>;
  /** Where the engine is listening, for an external tool. */
  engineAddress(): Promise<string>;
  /** What the engine can currently do. */
  capabilities(): Promise<Capabilities>;

  /** Models the engine offers to install. */
  listModels(): Promise<CatalogModel[]>;

  /** Begin installing a model, returning the handle to follow it by. */
  installModel(id: string): Promise<Installation>;

  /** How far an installation has got. */
  installation(name: string): Promise<Installation>;

  /** Stop an installation, keeping its partial file so a retry resumes. */
  cancelInstallation(name: string): Promise<Installation>;
  /** The schemas this build can create a collection from. */
  listPresets(): Promise<Preset[]>;
  /** Create a collection from a preset. Resolves to its resource name. */
  createCollection(request: CreateCollectionRequest): Promise<string>;
  /** Write a batch of text rows into one collection. */
  importPoints(request: ImportRequest): Promise<ImportResponse>;
  /** The points of one collection. */
  listPoints(collection: string): Promise<PointRow[]>;

  /** Every model host this build knows, and whether each is ready to use. */
  listProviders(): Promise<Provider[]>;

  /** Store a provider's credential, replacing any already there. */
  storeProviderKey(id: string, credential: string): Promise<void>;

  /** Forget a provider's credential. Forgetting an absent one is not an error. */
  forgetProviderKey(id: string): Promise<void>;

  /**
   * Read a credential so this window can call the provider.
   *
   * The one call that puts a key in the webview. Kept explicit rather than folded
   * into {@link listProviders} so the exposure is a call site that can be found —
   * and so a future engine-side proxy has one place to remove.
   */
  providerCredential(id: string): Promise<string>;

  /** Every organization this engine holds, soft-deleted ones included. */
  listOrganizations(): Promise<Organization[]>;

  /** Create an organization. `id` becomes the last segment of its name. */
  createOrganization(id: string, displayName: string): Promise<Organization>;

  /** Soft-delete an organization. It remains until its expiry passes. */
  deleteOrganization(name: string): Promise<Organization>;

  /** Restore a soft-deleted organization. */
  undeleteOrganization(name: string): Promise<Organization>;

  /** Every project under one organization. */
  listProjects(parent: string): Promise<Project[]>;

  /** Create a project under one organization. */
  createProject(parent: string, id: string, displayName: string): Promise<Project>;

  /** Every space under one organization. */
  listSpaces(parent: string): Promise<Space[]>;

  /**
   * Create a space, declaring its protection.
   *
   * Protection is required and cannot be changed later: it decides which
   * segments the contents are routed to.
   */
  createSpace(
    parent: string,
    id: string,
    displayName: string,
    protection: Protection,
  ): Promise<Space>;
}
