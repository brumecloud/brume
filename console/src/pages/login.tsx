import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useLoginError } from "@/hooks/useLoginError";
import Cookies from "js-cookie";
import { useState } from "react";

export const Login = () => {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");

  useLoginError();

  const submit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();

    fetch("http://localhost:9877/login", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        email,
        password,
      }),
    })
      .then((res) => res.text())
      .then((token) => {
        Cookies.set("access_token", token);
        window.location.pathname = "/";
      });
  };

  return (
    <main className="flex h-screen w-screen flex-col items-center justify-center">
      <h1>Login to Brume Cloud</h1>
      <form className="mt-4 flex w-[300px] flex-col gap-y-2" onSubmit={(e) => submit(e)}>
        <Input placeholder="Email" type="email" onChange={(e) => setEmail(e.target.value)} />
        <Input placeholder="Password" type="password" onChange={(e) => setPassword(e.target.value)} />
        <Button className="mt-2 w-full" type="submit">
          Login
        </Button>
      </form>
    </main>
  );
};
