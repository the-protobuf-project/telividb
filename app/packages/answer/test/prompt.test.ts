/** What the model is told, checked without a network or a key. */

import { describe, expect, test } from "bun:test";
import { SYSTEM, buildUser } from "../src/prompt";
import type { Passage } from "../src/types";

const passage = (id: string, text: string, score = 0.9): Passage => ({
  id,
  text,
  score,
});

describe("SYSTEM", () => {
  test("demands citations, because the panel shows the passages to check them against", () => {
    expect(SYSTEM).toContain("Cite");
  });

  test("permits an admission of ignorance", () => {
    // The failure this pipeline makes easy is a fluent answer built from
    // nothing. If this line is ever dropped the model will oblige.
    expect(SYSTEM.toLowerCase()).toContain("do not guess");
  });
});

describe("buildUser", () => {
  test("numbers passages from one, matching what the panel renders", () => {
    const prompt = buildUser("who?", [
      passage("a", "Ada wrote the first program."),
      passage("b", "Grace found the first bug."),
    ]);
    expect(prompt).toContain("[1] Ada wrote the first program.");
    expect(prompt).toContain("[2] Grace found the first bug.");
  });

  test("puts the question last", () => {
    const prompt = buildUser("who?", [passage("a", "text")]);
    expect(prompt.trimEnd().endsWith("Question: who?")).toBe(true);
  });

  test("says so when retrieval found nothing, rather than sending an empty list", () => {
    // An empty "Passages:" heading reads as though the passages were blank,
    // which invites the model to invent. This states the situation instead.
    const prompt = buildUser("who?", []);
    expect(prompt).toContain("No passages were retrieved");
    expect(prompt).toContain("Question: who?");
  });

  test("does not leak a score into the prompt", () => {
    // Scores are for the reader's judgement, not the model's. A similarity in
    // the prompt is a number the model will try to reason about.
    const prompt = buildUser("who?", [passage("a", "text", 0.8123)]);
    expect(prompt).not.toContain("0.8123");
  });
});
