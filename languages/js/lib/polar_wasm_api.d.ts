/* tslint:disable */
/* eslint-disable */
/**
*/
export class Polar {
  free(): void;
/**
*/
  constructor();
/**
* @param {string} src
* @param {string | undefined} filename
*/
  loadFile(src: string, filename?: string): void;
/**
* @param {string} name
* @param {string} value
*/
  registerConstant(name: string, value: string): void;
/**
* @returns {Query | undefined}
*/
  nextInlineQuery(): Query | undefined;
/**
* @param {string} src
* @returns {Query}
*/
  newQueryFromStr(src: string): Query;
/**
* @param {string} value
* @returns {Query}
*/
  newQueryFromTerm(value: string): Query;
/**
* @returns {BigInt}
*/
  newId(): BigInt;
}
/**
*/
export class Query {
  free(): void;
/**
* @returns {any}
*/
  nextEvent(): any;
/**
* @param {BigInt} call_id
* @param {string | undefined} value
*/
  callResult(call_id: BigInt, value?: string): void;
/**
* @param {BigInt} call_id
* @param {boolean} result
*/
  questionResult(call_id: BigInt, result: boolean): void;
/**
* @param {string} command
*/
  debugCommand(command: string): void;
/**
* @param {string} msg
*/
  appError(msg: string): void;
}
