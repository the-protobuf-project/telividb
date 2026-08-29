/**
 * Class-name composition for the component library.
 *
 * shadcn components import this by name; it is their one shared helper. `clsx`
 * resolves conditionals, and `twMerge` settles conflicts in favour of the last
 * class — so a caller passing `p-2` to a component that already sets `p-4` gets
 * `p-2` rather than whichever the stylesheet happened to order last.
 */

import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

/** Compose class names, resolving Tailwind conflicts. */
export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}

/**
 * Prop-shape helpers the component library expects.
 *
 * These are shadcn-svelte's own vocabulary, normally written by its `init`.
 * They exist because a Svelte component that forwards a DOM element, or
 * replaces its children with a snippet, has a prop shape the underlying
 * element's own types do not describe.
 */

/** Props with `children` removed — for a component that renders its own. */
export type WithoutChildren<T> = T extends { children?: unknown }
  ? Omit<T, "children">
  : T;

/** Props with `child` removed — for a component that will not delegate. */
export type WithoutChild<T> = T extends { child?: unknown }
  ? Omit<T, "child">
  : T;

/** Props with neither, for a component that owns its whole subtree. */
export type WithoutChildrenOrChild<T> = WithoutChildren<WithoutChild<T>>;

/**
 * Props plus a bindable `ref` to the rendered element.
 *
 * The escape hatch for measuring or focusing a node the component owns —
 * without it a caller would have to wrap the component in a div to reach the
 * DOM, which changes the layout to observe it.
 */
export type WithElementRef<T, U extends HTMLElement = HTMLElement> = T & {
  ref?: U | null;
};
