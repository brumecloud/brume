// API responses arrive as `unknown`: check their shape before typing them.

export class DecodeError extends Error {
  override name = "DecodeError";
}

export function decodeRecord(raw: unknown, what: string): Record<string, unknown> {
  if (typeof raw !== "object" || raw === null || Array.isArray(raw)) {
    throw new DecodeError(`${what}: expected an object`);
  }
  return raw as Record<string, unknown>;
}

export function decodeArray<T>(raw: unknown, decodeItem: (item: unknown) => T, what: string): T[] {
  if (!Array.isArray(raw)) {
    throw new DecodeError(`${what}: expected an array`);
  }
  return raw.map(decodeItem);
}

export function readString(record: Record<string, unknown>, key: string, what: string): string {
  const value = record[key];
  if (typeof value !== "string") {
    throw new DecodeError(`${what}.${key}: expected a string`);
  }
  return value;
}

export function readOneOf<const T extends string>(
  record: Record<string, unknown>,
  key: string,
  values: readonly T[],
  what: string,
): T {
  const value = record[key];
  if (!values.includes(value as T)) {
    throw new DecodeError(`${what}.${key}: expected one of ${values.join(", ")}`);
  }
  return value as T;
}
