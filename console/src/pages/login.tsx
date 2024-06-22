import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

export const Login = () => {
  return (
    <main className="flex h-screen w-screen flex-col items-center justify-center">
      <h1>Login to Brume Cloud</h1>
      <div className="mt-4 flex w-[300px] flex-col gap-y-2">
        <Input placeholder="Email" type="email" />
        <Input placeholder="Password" type="password" />
        <Button className="mt-2">Login</Button>
      </div>
    </main>
  );
};
