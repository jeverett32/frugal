export const MAX_USERS: number = 100;

/** Represents an identifier. */
export type Id = string;

/** User profile. */
export interface User {
  id: Id;
  name: string;
  greet(): string;
}

/** Server status. */
export enum Status {
  Active,
  Inactive,
}

/** Fetches a user. */
export function fetchUser(id: Id): Promise<User> {
  return Promise.resolve({ id, name: "test" } as User);
}

/** Basic repository. */
export class Repository<T> {
  private items: T[] = [];

  constructor(initial: T[] = []) {
    this.items = initial;
  }

  add(item: T): void {
    this.items.push(item);
  }

  get size(): number {
    return this.items.length;
  }
}

declare function log(message: string): void;
