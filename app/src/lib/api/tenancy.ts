/**
 * Organizations, projects and spaces.
 *
 * These mirror the Rust DTOs in `telividb-desktop-ipc` field for field, because
 * Tauri serializes those as JSON and this is the other end of it.
 *
 * Resource names arrive whole and are used whole. `organizations/acme` is the
 * only portable identity the server accepts — a window that split one apart and
 * rebuilt it would be reimplementing a format the server already owns, and would
 * be wrong the first time a segment contained something unexpected.
 */

import type { Protection } from "@telividb/answer";

/** A tenant: the top of the resource hierarchy. */
export interface Organization {
  /** Resource name, `organizations/{organization}`. */
  name: string;
  /** What a person calls it. */
  displayName: string;
  /** How many projects it holds. */
  projectCount: number;
  /** How many spaces it holds. */
  spaceCount: number;
  /** Whether it is soft-deleted and awaiting purge. */
  deleted: boolean;
}

/** A unit of work inside an organization. */
export interface Project {
  /** Resource name, `organizations/{organization}/projects/{project}`. */
  name: string;
  /** What a person calls it. */
  displayName: string;
  /** Whether it is soft-deleted and awaiting purge. */
  deleted: boolean;
}

/**
 * A protection boundary, which may span several projects.
 *
 * A sibling of a project rather than a child — the resource name is
 * `organizations/{organization}/spaces/{space}` and `projects` are references.
 * Protection is where secrecy lives, and secrecy does not follow the work
 * breakdown.
 */
export interface Space {
  /** Resource name, `organizations/{organization}/spaces/{space}`. */
  name: string;
  /** What a person calls it. */
  displayName: string;
  /** Projects this space serves, by resource name. */
  projects: string[];
  /** How it is protected. Declared at creation and never changed. */
  protection: Protection;
  /** Whether its key is currently unavailable. */
  locked: boolean;
  /** Whether it is soft-deleted and awaiting purge. */
  deleted: boolean;
}

/** The last segment of a resource name — what a person typed to create it. */
export function resourceId(name: string): string {
  return name.slice(name.lastIndexOf("/") + 1);
}

/**
 * Turn a display name into a resource id.
 *
 * Lowercased, non-alphanumerics folded to hyphens, trimmed. The server has the
 * final say — this only spares a person from typing the same thing twice.
 */
export function suggestId(displayName: string): string {
  return displayName
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 63);
}
