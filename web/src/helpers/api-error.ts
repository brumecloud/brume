const MAX_MESSAGE_LENGTH = 300;

export function apiErrorMessage(status: number, body: unknown): string {
  const message =
    typeof body === "object" && body !== null && "message" in body ? body.message : undefined;
  if (typeof message === "string" && message) {
    return message.slice(0, MAX_MESSAGE_LENGTH);
  }
  return `Erreur ${status} du serveur.`;
}
