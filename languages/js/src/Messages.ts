enum MessageKind {
  Print = 'Print',
  Warning = 'Warning',
}

interface Message {
  kind: MessageKind;
  msg: string;
}

export function processMessage(message: Message) {
  switch (message.kind) {
    case MessageKind.Print:
      console.log(message.msg);
      break;
    case MessageKind.Warning:
      console.warn(message.msg);
      break;
  }
}
