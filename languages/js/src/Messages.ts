export interface Message {
  kind: String;
  msg: String;
}

export function processMessage(message: Message) {
  if (message.kind === 'Print') {
    console.log(message.msg);
  } else if (message.kind === 'Warning') {
    console.warn(message.msg);
  }
}
