// Server ids: UUIDs today, bounded so a crafted URL cannot smuggle anything else.
const ID = /^[A-Za-z0-9_-]{1,64}$/;

export function isId(raw: string): boolean {
  return ID.test(raw);
}
