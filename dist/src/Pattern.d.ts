import { Dict } from './types';
/** Polar pattern. */
export declare class Pattern {
    readonly tag?: string;
    readonly fields: Dict;
    constructor({ tag, fields }: {
        tag?: string;
        fields: Dict;
    });
}
