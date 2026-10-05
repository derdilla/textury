import * as Y from 'yjs';

/// Utility to access properties of an element in the crdt files array.
export class FilesEntry {
  readonly e: Y.Map<unknown>;

  constructor(entry: Y.Map<unknown>) {
    this.e = entry;
  }

  static tryNew(entry: unknown): FilesEntry | 'corrupt' {
    if (!(entry instanceof Y.Map)) {
      console.warn(`Corrupted file: ${entry}`);
      return 'corrupt';
    }
    return new FilesEntry(entry);
  }

  get version(): string | 'corrupt' {
    const version = this.e.get('version');
    if (!this.#isString(version)) {
      console.warn(`Corrupt version "${version}" in ${this.e}`);
      return 'corrupt';
    }
    return version.toString();
  }

  get uuid(): string | 'corrupt' {
    const uuid = this.e.get('uuid');
    if (!this.#isString(uuid)) {
      console.warn(`Corrupt uuid "${uuid}" in ${this.e}`);
      return 'corrupt';
    }
    return uuid.toString();
  }

  get type(): ProjectStateFileType | 'corrupt' {
    const type = this.e.get('type');
    if (!this.#isString(type)) {
      console.warn(`Corrupt type "${type}" in ${this.e}`);
      return 'corrupt';
    }
    if (type != "File" && type != "Directory" && type != "Asset") {
      console.warn(`Unrecognized ProjectStateFileType: "${type}" in ${this.e}`);
      return 'corrupt';
    }
    return type as ProjectStateFileType;
  }

  get fullPath(): Y.Text | 'corrupt' {
    const fullPath = this.e.get('full_path');
    if (!(fullPath instanceof Y.Text)) {
      console.warn(`Corrupt full_path "${fullPath}" in ${this.e}`);
      return 'corrupt';
    }
    return fullPath;
  }

  get text(): Y.Text | null | 'corrupt' {
    const text = this.e.get('text');
    if (!text) return null;
    if (!(text instanceof Y.Text)) {
      console.warn(`Corrupt text "${text}" in ${this.e}`);
      return 'corrupt';
    }
    return text;
  }

  #isString(value: unknown): value is string | String {
    return typeof value === 'string' || value instanceof String;
  }
}

export type ProjectStateFileType = "File" | "Directory" | "Asset";
