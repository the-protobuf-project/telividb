/**
 * Turning a question and its passages into a prompt.
 *
 * Kept apart from the SDK adapters so the wording is one thing rather than four,
 * and so it can be tested without a network or a key.
 */

import type { Passage } from "./types";

/**
 * The instruction every provider gets.
 *
 * Two things it insists on. **Cite by number**, because an answer whose support
 * cannot be checked is indistinguishable from one that was invented — and this
 * window shows the passages beside the answer precisely so the reader can check.
 * **Say when the passages do not cover it**, because the failure this pipeline
 * makes easy is a fluent answer assembled from nothing, and a model asked to be
 * helpful will produce one unless told that admitting a gap is the better answer.
 */
export const SYSTEM = [
  "You answer from the passages given to you and from nothing else.",
  "",
  "Cite the passages you used by their number, like [1] or [2][4].",
  "If the passages do not contain the answer, say so plainly and stop —",
  "do not fill the gap from general knowledge, and do not guess.",
  "Be concise. Prefer the passages' own wording for anything specific.",
].join("\n");

/**
 * The question with its passages numbered above it.
 *
 * Numbered rather than bulleted so the citation instruction has something to
 * refer to, and the question repeated last because the final line is the one a
 * model weighs most.
 */
export function buildUser(question: string, passages: readonly Passage[]): string {
  if (passages.length === 0) {
    return [
      "No passages were retrieved for this question.",
      "",
      `Question: ${question}`,
    ].join("\n");
  }

  const numbered = passages
    .map((p, i) => `[${i + 1}] ${p.text}`)
    .join("\n\n");

  return [`Passages:`, ``, numbered, ``, `Question: ${question}`].join("\n");
}
