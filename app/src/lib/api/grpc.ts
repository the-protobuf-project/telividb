/**
 * The port, satisfied over gRPC-web.
 *
 * The browser half of the split: **inside Tauri the app speaks IPC, and on the
 * web it speaks gRPC.** That is not an implementation detail. The desktop build
 * links the engine into its own process, so IPC is a function call across a
 * thread boundary and there is no socket to secure; a browser has no engine and
 * must reach a daemon over the network.
 *
 * # Not implemented, and what it needs
 *
 * This adapter exists so the selection in `index.ts` is real code rather than a
 * comment about a build that might exist one day — and so the gap is visible at
 * the point someone would close it. Every method fails with the same sentence.
 *
 * Closing it needs three things, none of which are guesswork:
 *
 * 1. **Generated TypeScript stubs.** `buffers/protobuf` already generates Rust;
 *    the browser needs the same messages in TypeScript, which is another target
 *    in `buf.gen.yaml` rather than a hand-written client.
 * 2. **A gRPC-web transport.** Browsers cannot speak native gRPC — HTTP/2
 *    trailers are unreachable from `fetch` — which is why `telividb-server`
 *    already layers `tonic-web`.
 * 3. **An origin.** The desktop build knows where its engine is because it
 *    started it. A browser has to be told, and that is a deployment question:
 *    on Linux the engine runs as a daemon, the way `ollama` does.
 */

import type { CollectionSummary } from "./collection";
import type { Protection, Provider } from "@telividb/answer";
import type { Organization, Project, Space } from "./tenancy";
import type { SearchRequest, SearchResponse } from "./search";
import type { CreateCollectionRequest, Preset } from "./preset";
import type { ImportRequest, ImportResponse, PointRow } from "./import";
import type { Capabilities } from "./engine";
import type { CatalogModel, Installation } from "./model";
import type { TelividbClient } from "./port";

/** What every method reports until the transport is built. */
const UNBUILT =
  "This page is running outside the desktop app, where the engine is reached " +
  "over gRPC-web. That transport is not built yet — run the desktop app, or " +
  "see api/grpc.ts for what it needs.";

/** A client reaching a telividb daemon over gRPC-web. */
export class GrpcWebClient implements TelividbClient {
  /**
   * Where the daemon is.
   *
   * Kept even though nothing reads it yet: the origin is the one piece of
   * configuration this adapter cannot infer, and naming it here is what makes
   * that obvious to whoever builds the transport.
   */
  public constructor(private readonly origin: string) {}

  /** The daemon this client is configured to reach. */
  public get endpoint(): string {
    return this.origin;
  }

  private unbuilt(): never {
    throw new Error(UNBUILT);
  }

  public async listCollections(): Promise<CollectionSummary[]> {
    this.unbuilt();
  }
  public async search(_request: SearchRequest): Promise<SearchResponse> {
    this.unbuilt();
  }
  public async engineAddress(): Promise<string> {
    this.unbuilt();
  }
  public async capabilities(): Promise<Capabilities> {
    this.unbuilt();
  }
  public async listPresets(): Promise<Preset[]> {
    this.unbuilt();
  }
  public async createCollection(_request: CreateCollectionRequest): Promise<string> {
    this.unbuilt();
  }
  public async importPoints(_request: ImportRequest): Promise<ImportResponse> {
    this.unbuilt();
  }
  public async listPoints(_collection: string): Promise<PointRow[]> {
    this.unbuilt();
  }
  public async listModels(): Promise<CatalogModel[]> {
    this.unbuilt();
  }
  public async installModel(_id: string): Promise<Installation> {
    this.unbuilt();
  }
  public async installation(_name: string): Promise<Installation> {
    this.unbuilt();
  }
  public async cancelInstallation(_name: string): Promise<Installation> {
    this.unbuilt();
  }
  public async listProviders(): Promise<Provider[]> {
    this.unbuilt();
  }
  public async storeProviderKey(_id: string, _credential: string): Promise<void> {
    this.unbuilt();
  }
  public async forgetProviderKey(_id: string): Promise<void> {
    this.unbuilt();
  }
  public async providerCredential(_id: string): Promise<string> {
    this.unbuilt();
  }
  public async listOrganizations(): Promise<Organization[]> {
    this.unbuilt();
  }
  public async createOrganization(
    _id: string,
    _displayName: string,
  ): Promise<Organization> {
    this.unbuilt();
  }
  public async deleteOrganization(_name: string): Promise<Organization> {
    this.unbuilt();
  }
  public async undeleteOrganization(_name: string): Promise<Organization> {
    this.unbuilt();
  }
  public async listProjects(_parent: string): Promise<Project[]> {
    this.unbuilt();
  }
  public async createProject(
    _parent: string,
    _id: string,
    _displayName: string,
  ): Promise<Project> {
    this.unbuilt();
  }
  public async listSpaces(_parent: string): Promise<Space[]> {
    this.unbuilt();
  }
  public async createSpace(
    _parent: string,
    _id: string,
    _displayName: string,
    _protection: Protection,
  ): Promise<Space> {
    this.unbuilt();
  }
}
