/**
 * Types of messages received from the Polar VM that should be relayed to
 * library consumers.
 *
 * @internal
 */
enum MessageKind {
  Print = 'Print',
  Warning = 'Warning',
}

/**
 * JSON payload containing a message emitted by the Polar VM.
 *
 * @internal
 */
interface Message {
  kind: MessageKind;
  msg: string;
}

/**
 * Relay messages received from the Polar VM to library consumers.
 *
 * @internal
 */
export function processMessage(message: Message) {
  switch (message.kind) {
    case MessageKind.Print:
      console.log(message.msg);
      break;
    case MessageKind.Warning:
      console.warn('[warning]', message.msg);
      break;
  }
}
