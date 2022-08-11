"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.processMessage = void 0;
/**
 * Types of messages received from the Polar VM that should be relayed to
 * library consumers.
 *
 * @internal
 */
var MessageKind;
(function (MessageKind) {
    MessageKind["Print"] = "Print";
    MessageKind["Warning"] = "Warning";
})(MessageKind || (MessageKind = {}));
/**
 * Relay messages received from the Polar VM to library consumers.
 *
 * @internal
 */
function processMessage(message) {
    switch (message.kind) {
        case MessageKind.Print:
            console.log(message.msg);
            break;
        case MessageKind.Warning:
            console.warn('[warning]', message.msg);
            break;
    }
}
exports.processMessage = processMessage;
//# sourceMappingURL=messages.js.map