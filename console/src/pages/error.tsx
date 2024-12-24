import { useRouteError } from "react-router-dom";

export function ErrorPage() {
  const error = useRouteError();
  return (
    <div className="flex h-screen w-full flex-col items-center justify-center">
      <div className="flex flex-col gap-4">
        <div className="flex flex-col gap-2">
          <h1 className="text-2xl font-bold">
            An error has occurred
          </h1>
          <p className="text-sm text-gray-500">
            An error occurred while loading this page. Brume
            engineering has been informed.
          </p>
        </div>
        <div className="flex flex-col gap-2">
          <h2 className="text-sm font-medium">Error Logs</h2>
          <div className="flex flex-col rounded-md bg-gray-800 p-4">
            <pre className="max-h-[300px] overflow-y-auto text-sm text-white">
              {error instanceof Error
                ? error.message
                : "No error message to show"}
            </pre>
          </div>
        </div>
      </div>
    </div>
  );
}
