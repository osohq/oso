import type { Connection, SelectQueryBuilder } from 'typeorm';
import { Adapter } from './filter';
export declare function typeOrmAdapter<R>(connection: Connection): Adapter<SelectQueryBuilder<R>, R>;
