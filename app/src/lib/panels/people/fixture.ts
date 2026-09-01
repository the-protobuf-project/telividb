/**
 * Sample people, until the `Identity` service is served.
 *
 * Real-shaped rather than lorem: resource names in the form the engine issues
 * (`users/{user}`, `userGroups/{group}`, and a binding scoped to an
 * organization or a project), and principals in the form an authenticated
 * request actually arrives with. The shapes are what this screen is *for* —
 * it is the schema for a permission system that does not exist yet, and a
 * fixture with the wrong shape would make the wrong screen look right.
 *
 * This module is the seam. When `Identity.ListUsers` lands, the panel imports
 * the client instead and the `sample data` tag comes off.
 */

/** Someone who can be granted access. */
export interface Person {
  /** Resource name, `users/{user}`. */
  readonly name: string;
  /** What a person calls them. */
  readonly displayName: string;
  /** The identity an authenticated request arrives with. */
  readonly principal: string;
  /** Groups they belong to, by resource name. */
  readonly groups: readonly string[];
}

/** A named set of users, and what a role is actually granted to. */
export interface Group {
  /** Resource name, `userGroups/{group}`. */
  readonly name: string;
  /** What a person calls it. */
  readonly displayName: string;
}

/** A role granted to a group over some scope. */
export interface Binding {
  /** Resource name of the binding itself. */
  readonly name: string;
  /** The group it grants to. */
  readonly group: string;
  /** The role granted. */
  readonly role: string;
  /** What the grant covers — an organization, a project, or a space. */
  readonly scope: string;
}

/** The groups. */
export const GROUPS: readonly Group[] = [
  { name: "userGroups/engineering", displayName: "Engineering" },
  { name: "userGroups/research", displayName: "Research" },
  { name: "userGroups/finance", displayName: "Finance" },
];

/** The people. */
export const PEOPLE: readonly Person[] = [
  {
    name: "users/srikanth",
    displayName: "Srikanth Kandarp",
    principal: "srikanth@the-protobuf-project.org",
    groups: ["userGroups/engineering", "userGroups/research"],
  },
  {
    name: "users/ada",
    displayName: "Ada Okonkwo",
    principal: "ada@the-protobuf-project.org",
    groups: ["userGroups/engineering"],
  },
  {
    name: "users/mira",
    displayName: "Mira Halvorsen",
    principal: "mira@the-protobuf-project.org",
    groups: ["userGroups/research"],
  },
  {
    name: "users/tomas",
    displayName: "Tomás Ferreira",
    // A user that exists as a record with no credential behind it yet, which is
    // a real state: someone invited but not arrived.
    principal: "",
    groups: ["userGroups/finance"],
  },
];

/** The grants. */
export const BINDINGS: readonly Binding[] = [
  {
    name: "organizations/acme/roleBindings/eng-writer",
    group: "userGroups/engineering",
    role: "roles/writer",
    scope: "organizations/acme/projects/retrieval",
  },
  {
    name: "organizations/acme/roleBindings/research-reader",
    group: "userGroups/research",
    role: "roles/reader",
    scope: "organizations/acme",
  },
  {
    name: "organizations/acme/roleBindings/finance-owner",
    group: "userGroups/finance",
    role: "roles/owner",
    scope: "organizations/acme/spaces/board",
  },
];
