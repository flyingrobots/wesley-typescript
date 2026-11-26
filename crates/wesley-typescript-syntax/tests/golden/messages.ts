/** A run started. */
export interface Started {
    event: "started";
    /** When the run started. */
    at: string;
}
/** A run finished. */
export interface Finished {
    event: "finished";
    code: number;
}
/** Every event name. */
export type EventName = "started" | "finished";
/** Any message, told apart by `event`. */
export type Message = Started | Finished;
