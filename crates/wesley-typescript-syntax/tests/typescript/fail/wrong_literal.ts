// expect-error: TS2322
// A literal that is not one of the union's events does not type-check.
import type { Message } from "../../golden/messages";

export const wrong: Message = { event: "stopped", code: 1 };
