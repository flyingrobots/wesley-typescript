// A string-literal `event` field narrows the generated union.
import type { EventName, Message } from "../../golden/messages";
import type { Run } from "../../golden/every_variant";

export function exitCode(message: Message): number | undefined {
    if (message.event === "finished") {
        const code: number = message.code;
        return code;
    }
    const at: string = message.at;
    return at.length > 0 ? undefined : undefined;
}

export const names: EventName[] = ["started", "finished"];

export function command(run: Run): string[] | undefined {
    return run["what-ran"]?.argv;
}
