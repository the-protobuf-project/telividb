/**
 * The command names the Rust side registers.
 *
 * Extracted so both halves of the Tauri client name the same constants. A string
 * literal repeated at a call site is a rename that compiles and then fails at
 * runtime, which is the one failure mode this bridge cannot type-check away.
 *
 * An object rather than a `const enum`: this build sets `isolatedModules`, under
 * which TypeScript does not support `const enum` at all — each file is
 * transpiled alone, so the values cannot be inlined across the module boundary.
 * It happens to work through esbuild's own handling, which is not a guarantee to
 * build a bridge on. `as const` gives the same literal types with none of that.
 */

export const Command = {
  ListCollections: "list_collections",
  Search: "search",
  EngineAddress: "engine_address",
  Capabilities: "capabilities",
  ListPresets: "list_presets",
  CreateCollection: "create_collection",
  ImportPoints: "import_points",
  ListPoints: "list_points",
  ListModels: "list_models",
  InstallModel: "install_model",
  Installation: "installation",
  CancelInstallation: "cancel_installation",
  ListProviders: "list_providers",
  StoreProviderKey: "store_provider_key",
  ForgetProviderKey: "forget_provider_key",
  ProviderCredential: "provider_credential",
  ListOrganizations: "list_organizations",
  CreateOrganization: "create_organization",
  DeleteOrganization: "delete_organization",
  UndeleteOrganization: "undelete_organization",
  ListProjects: "list_projects",
  CreateProject: "create_project",
  ListSpaces: "list_spaces",
  CreateSpace: "create_space",
} as const;

/** One of the command names above. */
export type Command = (typeof Command)[keyof typeof Command];
