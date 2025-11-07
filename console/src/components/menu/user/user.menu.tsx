import { useQuery } from "@apollo/client";
import { BsThreeDotsVertical } from "react-icons/bs";
import { FaRegCreditCard } from "react-icons/fa";
import { IoNotifications } from "react-icons/io5";
import { LuCircleUserRound } from "react-icons/lu";
import { TbLogout } from "react-icons/tb";
import { NavLink } from "react-router-dom";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { ME_QUERY } from "@/gql/user.graphql";

export const UserMenu = () => {
  const { data: me, loading } = useQuery(ME_QUERY, {
    fetchPolicy: "cache-first",
  });

  if (loading) {
    return (
      <div className="flex animate-pulse select-none flex-row items-center gap-x-3 transition-all">
        <div className="h-[30px] w-[30px] rounded-full bg-gray-200" />
        <div className="h-[12px] w-[130px] rounded-full bg-gray-200" />
      </div>
    );
  }

  if (!me?.me) {
    return null;
  }

  const user = me.me;

  return (
    <div className="flex select-none flex-row items-center justify-between gap-x-3">
      <div className="flex flex-row items-center gap-2">
        <Avatar className="h-8 w-8">
          <AvatarImage src={user.avatar} />
          <AvatarFallback>{user.name.slice(0, 2)}</AvatarFallback>
        </Avatar>
        <span className="text-gray-800 text-sm">{user.name}</span>
      </div>
      <DropdownMenu>
        <DropdownMenuTrigger className="focus:outline-none focus:ring-0">
          <div className="w-full">
            <BsThreeDotsVertical />
          </div>
        </DropdownMenuTrigger>
        <DropdownMenuContent
          className="w-(--radix-dropdown-menu-trigger-width) min-w-56 rounded-lg"
          side="right"
          align="end"
          sideOffset={4}
        >
          <DropdownMenuLabel className="p-0 font-normal">
            <div className="flex items-center gap-2 px-1 py-1.5 text-left text-sm">
              <Avatar className="h-8 w-8 rounded-lg">
                <AvatarImage src={user.avatar} alt={user.name} />
                <AvatarFallback className="rounded-lg">CN</AvatarFallback>
              </Avatar>
              <div className="grid flex-1 text-left text-sm leading-tight">
                <span className="truncate font-medium">{user.name}</span>
                <span className="truncate text-muted-foreground text-xs">
                  admin@brume.dev
                </span>
              </div>
            </div>
          </DropdownMenuLabel>
          <DropdownMenuSeparator />
          <DropdownMenuGroup>
            <DropdownMenuItem>
              <LuCircleUserRound className="mr-2" />
              Account
            </DropdownMenuItem>
            <DropdownMenuItem>
              <FaRegCreditCard className="mr-2" />
              Billing
            </DropdownMenuItem>
            <DropdownMenuItem>
              <IoNotifications className="mr-2" />
              Notifications
            </DropdownMenuItem>
          </DropdownMenuGroup>
          <DropdownMenuSeparator />
          <DropdownMenuItem>
            <NavLink to="/logout" className="flex w-full flex-row items-center">
              <TbLogout className="mr-2" />
              Log out
            </NavLink>
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
};
