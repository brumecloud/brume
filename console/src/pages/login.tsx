import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useLoginError } from "@/hooks/useLoginError";
import { ENV } from "@/utils/env";

export const Login = () => {
	const [state, setState] = useState<"normal" | "magic-link">("normal");
	const [email, setEmail] = useState("");
	const [code, setCode] = useState("");
	const navigate = useNavigate();

	useLoginError();

	const submit = (e: React.FormEvent<HTMLFormElement>) => {
		e.preventDefault();

		fetch(`${ENV.ORCHESTRATOR_URL}/wos/magic-link?email=${email}`, {
			method: "GET",
		}).then(() => setState("magic-link"));
	};

	const submitCode = (e: React.FormEvent<HTMLFormElement>) => {
		e.preventDefault();

		fetch(
			`${ENV.ORCHESTRATOR_URL}/wos/magic-code?code=${code}&email=${email}`,
			{
				method: "GET",
				credentials: "include",
			},
		).then(() => {
			// fetch(`${ENV.ORCHESTRATOR_URL}/wos/me`, {
			// 	credentials: "include",
			// }).then(() => {
			// 	navigate("/");
			// });
		});
	};

	return (
		<main className="flex h-screen w-screen flex-col items-center justify-center">
			<h1>Login to Brume Cloud</h1>
			{state === "normal" && (
				<form
					className="mt-4 flex w-[300px] flex-col gap-y-2"
					onSubmit={(e) => submit(e)}
				>
					<Input
						placeholder="Email"
						type="email"
						onChange={(e) => setEmail(e.target.value)}
					/>
					<Button className="mt-2 w-full" type="submit" disabled={!email}>
						Send me a magic link
					</Button>
				</form>
			)}
			{state === "magic-link" && (
				<form
					className="mt-4 flex w-[300px] flex-col gap-y-2"
					onSubmit={(e) => submitCode(e)}
				>
					<Input
						placeholder="Code"
						type="text"
						onChange={(e) => setCode(e.target.value)}
					/>
					<Button className="mt-2 w-full" type="submit" disabled={!code}>
						Login
					</Button>
				</form>
			)}
		</main>
	);
};
