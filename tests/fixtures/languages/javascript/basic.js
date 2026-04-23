const MAX_RETRIES = 3;

/**
 * Loads an item.
 */
function loadItem(id) {
  return fetch(`/items/${id}`);
}

/**
 * Represents a repository.
 */
class Repository {
  constructor(name) {
    this.name = name;
  }

  /** Save the repo. */
  save() {
    return true;
  }

  static create(name) {
    return new Repository(name);
  }
}

export const helper = (value) => value + 1;

export default function main() {
  return 0;
}
