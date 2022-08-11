/**
 * Types of messages received from the Polar VM that should be relayed to
 * library consumers.
 *
 * @internal
 */
declare enum MessageKind {
    Print = "Print",
    Warning = "Warning"
}
/**
 * JSON payload containing a message emitted by the Polar VM.
 *
 * @internal
 */
export interface Message {
    kind: MessageKind;
    msg: string;
}
/**
 * Relay messages received from the Polar VM to library consumers.
 *
 * @internal
 */
export declare function processMessage(message: Message): void;
export {};
