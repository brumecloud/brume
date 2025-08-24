import { REGEXP_ONLY_DIGITS } from "input-otp";
import { ArrowRightIcon, Loader2Icon } from "lucide-react";
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

		// fetch(`${ENV.ORCHESTRATOR_URL}/wos/magic-link?email=${email}`, {
		// 	method: "GET",
		// }).then(() => setState("magic-link"));
	};

	const submitCode = (e: React.FormEvent<HTMLFormElement>) => {
		e.preventDefault();

		setLoading(true);

		fetch(
			`${ENV.ORCHESTRATOR_URL}/wos/magic-code?code=${code}&email=${email}`,
			{
				method: "GET",
				credentials: "include",
			},
		).then(() => {
			navigate("/overview");
			setLoading(false);
		});
	};

	return (
		<main className="flex h-screen w-screen flex-col items-center justify-center bg-[size:180px] [z-index:-1] bg-repeat">
			<div className="flex flex-row border bg-white rounded-lg h-[500px] w-[800px] overflow-hidden z-10 shadow-lg shadow-gray-900/10">
				<div className="p-2 h-full rounded-r-sm overflow-hidden w-1/2">
					<img
						src={login}
						alt="Brume Cloud"
						className="rounded-sm object-cover w-full h-full"
					/>
				</div>
				<div className="flex flex-col gap-4 p-4 pr-6 justify-between rounded-sm w-1/2">
					<div className="h-full flex flex-col items-center justify-center gap-4">
						<div className="w-full">
							<h1 className="text-lg font-semibold">Login to Brume Cloud</h1>
							<h1 className="text-sm text-gray-500">
								If you already have an account, you can login using a magic link
							</h1>
						</div>
						{state === "normal" && (
							<form
								className="mt-4 flex w-full flex-col gap-y-2"
								onSubmit={(e) => submit(e)}
							>
								<Input
									placeholder="Email"
									type="email"
									onChange={(e) => setEmail(e.target.value)}
								/>
								<Button
									className="mt-2 w-fit"
									type="submit"
									disabled={!email || loading === true}
								>
									{loading && <Spinner className="border-white" />}
									Send me a magic link
								</Button>
							</form>
						)}
						{state === "magic-link" && (
							<form
								className="mt-4 flex w-full justify-center flex-col gap-y-2"
								onSubmit={(e) => submitCode(e)}
							>
								<h2 className="text-sm text-gray-500 italic">
									Enter the code sent to your email
								</h2>
								<InputOTP
									className="text-gray-800 z-10"
									maxLength={6}
									pattern={REGEXP_ONLY_DIGITS}
									onChange={(e) => setCode(e)}
									onComplete={(e) => setCode(e)}
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
									type="submit"
									disabled={!code || loading === true}
								>
									{loading && <Spinner />}
									Login
								</Button>
							</form>
						)}
					</div>
					<div className="h-[150px] flex flex-col gap-4 justify-center">
						<div className="w-full h-px bg-gray-200" />
						<div className="flex flex-col gap-y-2 ">
							<div className="flex flex-row gap-x-2 items-center">
								<Input
									placeholder="join the waitlist"
									type="email"
									onChange={(e) => setEmail(e.target.value)}
								/>
								<Button className="size-9" type="submit" disabled={!email}>
									<ArrowRightIcon className="w-4 h-4" />
								</Button>
							</div>
							<h1 className="m-auto text-xs text-gray-500">
								we are still in early alpha, but you can join the waitlist
							</h1>
						</div>
					</div>
				</div>
			</div>
			<div
				style={{
					backgroundImage: `url(${noise})`,
				}}
				className="absolute top-0 left-0 flex h-screen w-screen flex-col items-center justify-center bg-[size:180px] [z-index:-1] bg-repeat opacity-[0.04]"
			/>
		</main>
	);
};
