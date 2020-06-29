/* tslint:disable */
/* eslint-disable */
/**
*/
export enum QueryEvent {
  None,
  Debug,
  Done,
  MakeExternal,
  ExternalCall,
  ExternalIsa,
  ExternalIsSubSpecializer,
  Result,
}
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
* @returns {Query | undefined} 
*/
  nextInlineQuery(): Query | undefined;
/**
* @param {string} src 
* @returns {Query} 
*/
  newQueryFromStr(src: string): Query;
/**
* @param {Term} term 
* @returns {Query} 
*/
  newQueryFromTerm(term: Term): Query;
/**
* @returns {Query} 
*/
  newQueryFromRepl(): Query;
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
* @param {Term | undefined} value 
*/
  callResult(call_id: BigInt, value?: Term): void;
/**
* @param {BigInt} call_id 
* @param {boolean} result 
*/
  questionResult(call_id: BigInt, result: boolean): void;
/**
* @param {string} command 
*/
  debugCommand(command: string): void;
}
/**
* Represents a concrete instance of a Polar value
*/
export class Term {
  free(): void;
}
