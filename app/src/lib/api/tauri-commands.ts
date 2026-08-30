/**
 * The command names the Rust side registers.
 *
 * Extracted so both halves of the Tauri client name the same constants. A string
 * literal repeated at a call site is a rename that compiles and then fails at
 * runtime, which is the one failure mode this bridge cannot type-check away.
 */

export const enum Command {
  ListCollections = "list_collections",
  Search = "search",
  EngineAddress = "engine_address",
  Capabilities = "capabilities",
  ListPresets = "list_presets",
  CreateCollection = "create_collection",
  ImportPoints = "import_points",
  ListPoints = "list_points",
  ListModels = "list_models",
  InstallModel = "install_model",
  Installation = "installation",
  CancelInstallation = "cancel_installation",
  ListProviders = "list_providers",
  StoreProviderKey = "store_provider_key",
  ForgetProviderKey = "forget_provider_key",
  ProviderCredential = "provider_credential",
  ListOrganizations = "list_organizations",
  CreateOrganization = "create_organization",
  DeleteOrganization = "delete_organization",
  UndeleteOrganization = "undelete_organization",
  ListProjects = "list_projects",
  CreateProject = "create_project",
  ListSpaces = "list_spaces",
  CreateSpace = "create_space",
}
