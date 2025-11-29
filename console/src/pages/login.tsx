import { REGEXP_ONLY_DIGITS } from "input-otp";
import { ArrowRightIcon } from "lucide-react";
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import login from "@/assets/login2.png";
import noise from "@/assets/noise.png";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  InputOTP,
  InputOTPGroup,
  InputOTPSeparator,
  InputOTPSlot,
} from "@/components/ui/input-otp";
import { Spinner } from "@/components/ui/spinner";
import { useLoginError } from "@/hooks/useLoginError";
import { ENV } from "@/utils/env";

export const Login = () => {
  const [state, setState] = useState<"normal" | "magic-link">("normal");
  const [email, setEmail] = useState("");
  const [code, setCode] = useState("");
  const navigate = useNavigate();

  const [loading, setLoading] = useState<boolean | null>(null);

  useLoginError();

  const submit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();

    setLoading(true);

    fetch(`${ENV.ORCHESTRATOR_URL}/wos/magic-link?email=${email}`, {
      method: "GET",
    }).then(() => {
      setState("magic-link");
      setLoading(false);
    });
  };

  const submitCode = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();

    setLoading(true);

    fetch(
      `${ENV.ORCHESTRATOR_URL}/wos/magic-code?code=${code}&email=${email}`,
      {
        method: "GET",
        credentials: "include",
      }
    ).then(() => {
      navigate("/stacks");
      setLoading(false);
    });
  };

  return (
    <main className="flex h-screen w-screen flex-col items-center justify-center bg-[size:180px] bg-repeat [z-index:-1]">
      <div className="z-10 flex h-[500px] w-[800px] flex-row overflow-hidden rounded-lg border bg-white shadow-gray-900/10 shadow-lg">
        <div className="h-full w-1/2 overflow-hidden rounded-r-sm p-2">
          <img
            alt="Brume Cloud"
            className="h-full w-full rounded-sm object-cover"
            src={login}
          />
        </div>
        <div className="flex w-1/2 flex-col justify-between gap-4 rounded-sm p-4 pr-6">
          <div className="flex h-full flex-col items-center justify-center gap-4">
            <div className="w-full">
              <h1 className="font-semibold text-lg">Login to Brume Cloud</h1>
              <h1 className="text-gray-500 text-sm">
                If you already have an account, you can login using a magic link
              </h1>
            </div>
            {state === "normal" && (
              <form
                className="mt-4 flex w-full flex-col gap-y-2"
                onSubmit={(e) => submit(e)}
              >
                <Input
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="Email"
                  type="email"
                />
                <Button
                  className="mt-2 w-fit"
                  disabled={!email || loading === true}
                  type="submit"
                >
                  {loading && <Spinner className="border-white" size="sm" />}
                  Send me a magic link
                </Button>
              </form>
            )}
            {state === "magic-link" && (
              <form
                className="mt-4 flex w-full flex-col justify-center gap-y-2"
                onSubmit={(e) => submitCode(e)}
              >
                <h2 className="text-gray-500 text-sm italic">
                  Enter the code sent to your email
                </h2>
                <InputOTP
                  className="z-10 text-gray-800"
                  maxLength={6}
                  onChange={(e) => setCode(e)}
                  onComplete={(e) => {
                    setCode(e);
                    console.log(e);
                  }}
                  pattern={REGEXP_ONLY_DIGITS}
                >
                  <InputOTPGroup>
                    <InputOTPSlot index={0} />
                    <InputOTPSlot index={1} />
                    <InputOTPSlot index={2} />
                  </InputOTPGroup>
                  <InputOTPSeparator />
                  <InputOTPGroup>
                    <InputOTPSlot index={3} />
                    <InputOTPSlot index={4} />
                    <InputOTPSlot index={5} />
                  </InputOTPGroup>
                </InputOTP>
                <Button
                  className="mt-2 w-fit"
                  disabled={!code || loading === true}
                  type="submit"
                >
                  {loading && <Spinner size="sm" />}
                  Login
                </Button>
              </form>
            )}
          </div>
          <div className="flex h-[150px] flex-col justify-center gap-4">
            <div className="h-px w-full bg-gray-200" />
            <div className="flex flex-col gap-y-2">
              <div className="flex flex-row items-center gap-x-2">
                <Input
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="join the waitlist"
                  type="email"
                />
                <Button className="size-9" disabled={!email} type="submit">
                  <ArrowRightIcon className="h-4 w-4" />
                </Button>
              </div>
              <h1 className="m-auto text-gray-500 text-xs">
                we are still in early alpha, but you can join the waitlist
              </h1>
            </div>
          </div>
        </div>
      </div>
      <div
        className="absolute top-0 left-0 flex h-screen w-screen flex-col items-center justify-center bg-[size:180px] bg-repeat opacity-[0.04] [z-index:-1]"
        style={{
          backgroundImage: `url(${noise})`,
        }}
      />
    </main>
  );
};
