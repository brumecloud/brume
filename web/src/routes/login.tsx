import { createFileRoute, redirect } from "@tanstack/react-router";

import { loginState } from "~/api/session";
import { LoginPage } from "~/components/session/login-page";
import { parseLoginHash } from "~/helpers/session";

// `brume dev` prints a one-time link to this page, its token in the hash.
export const Route = createFileRoute("/login")({
  loader: async ({ location }) => {
    const state = await loginState(parseLoginHash(location.hash));
    if (state === "signed-in") {
      throw redirect({ to: "/projects", replace: true });
    }
    return { refused: state === "link-refused" };
  },
  component: LoginRoute,
});

function LoginRoute() {
  const { refused } = Route.useLoaderData();
  return <LoginPage refused={refused} />;
}
