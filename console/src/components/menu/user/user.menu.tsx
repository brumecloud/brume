import { useUser } from "@/hooks/useUser";

export const UserMenu = () => {
  const { me, loading } = useUser();

  if (loading || !me) {
    return (
      <div className="flex animate-pulse select-none flex-row items-center gap-x-3 transition-all">
        <div className="h-[30px] w-[30px] rounded-full bg-gray-200" />
        <div className="h-[12px] w-[130px] rounded-full bg-gray-200" />
      </div>
    );
  }

  return (
    <div className="flex select-none flex-row items-center gap-x-3">
      <img
        src={me.avatar}
        alt="avatar"
        width={30}
        height={30}
        className="rounded-full"
      />
      <span className="text-sm text-gray-800">{me.name}</span>
    </div>
  );
};
