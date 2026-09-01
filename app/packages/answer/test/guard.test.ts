/** The rule that protected content is answered locally or not at all. */

import { describe, expect, test } from "bun:test";
import { WouldLeaveMachine, mayAnswer } from "../src/guard";
import type { Protection } from "../src/guard";
import type { Provider } from "../src/types";

const provider = (id: string, locality: "local" | "remote"): Provider => ({
  id,
  displayName: id,
  locality,
  note: "",
  models: ["m"],
  credentialHint: "",
  configured: true,
});

const local = provider("ollama", "local");
const remote = provider("openai", "remote");

describe("mayAnswer", () => {
  test.each<Protection>(["vault", "sealed"])(
    "refuses a remote provider for a %s space",
    (protection) => {
      expect(() => mayAnswer("notes", protection, remote)).toThrow(
        WouldLeaveMachine,
      );
    },
  );

  test.each<Protection>(["vault", "sealed"])(
    "allows a local provider for a %s space",
    (protection) => {
      expect(() => mayAnswer("notes", protection, local)).not.toThrow();
    },
  );

  test.each<Protection>(["none", "private"])(
    "allows a remote provider for a %s space",
    (protection) => {
      // "private" is an owner predicate, not a cryptographic guarantee, so it
      // does not carry the local-only rule. Rule 25.
      expect(() => mayAnswer("notes", protection, remote)).not.toThrow();
    },
  );

  test("names the space and the provider, so the refusal is actionable", () => {
    try {
      mayAnswer("journal", "vault", remote);
      throw new Error("should have refused");
    } catch (cause) {
      expect(cause).toBeInstanceOf(WouldLeaveMachine);
      expect((cause as Error).message).toContain("journal");
      expect((cause as Error).message).toContain("openai");
    }
  });
});

describe("a local provider reached over the network", () => {
  // Ollama is the one provider a vault may use, and it is reached by address
  // rather than by key — an address that is stored and editable. Treating
  // `locality: "local"` as proof would let vault contents leave through exactly
  // the provider the rule exists to permit.
  const remoteHost = "http://ollama.example.com:11434";

  test.each<Protection>(["vault", "sealed"])(
    "is refused for a %s space when its endpoint is not loopback",
    (protection) => {
      expect(() => mayAnswer("notes", protection, local, remoteHost)).toThrow(
        WouldLeaveMachine,
      );
    },
  );

  test.each(["http://localhost:11434", "http://127.0.0.1:11434", "http://[::1]:11434", ""])(
    "is allowed for a vault when the endpoint is %s",
    (endpoint) => {
      expect(() => mayAnswer("notes", "vault", local, endpoint)).not.toThrow();
    },
  );

  test("treats an unparseable endpoint as remote", () => {
    // Failing closed: "I could not read it" is not evidence that it is safe.
    expect(() => mayAnswer("notes", "vault", local, "not a url")).toThrow(
      WouldLeaveMachine,
    );
  });

  test("does not restrict an unprotected space", () => {
    expect(() => mayAnswer("notes", "none", local, remoteHost)).not.toThrow();
  });
});
