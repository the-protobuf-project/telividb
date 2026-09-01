/** Answering: the question, the passages, and the model that writes the prose. */

export { answer } from "./answerer";
export { SYSTEM, buildUser } from "./prompt";
export { WouldLeaveMachine, mayAnswer, type Protection } from "./guard";
export { resolveFetch, type FetchLike } from "./transport";
export type { AnswerChunk, Ask, Locality, Passage, Provider } from "./types";
