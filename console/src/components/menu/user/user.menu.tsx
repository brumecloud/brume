import {
  Avatar,
  AvatarFallback,
  AvatarImage,
} from "@/components/ui/avatar";
import { USER_FRAGMENT } from "@/gql/user.graphql";
import { useFragment } from "@apollo/client";

export const UserMenu = () => {
  const { data: me, complete } = useFragment({
    fragment: USER_FRAGMENT,
    from: "me",
  });

  if (!complete) {
    return (
      <div className="flex animate-pulse select-none flex-row items-center gap-x-3 transition-all">
        <div className="h-[30px] w-[30px] rounded-full bg-gray-200" />
        <div className="h-[12px] w-[130px] rounded-full bg-gray-200" />
      </div>
    );
  }

  return (
    <div className="flex select-none flex-row items-center gap-x-3">
      <Avatar className="h-8 w-8">
        <AvatarImage src={me.avatar} />
        <AvatarFallback>{me.name.slice(0, 2)}</AvatarFallback>
      </Avatar>
      <span className="text-sm text-gray-800">{me.name}</span>
    </div>
  );
};
