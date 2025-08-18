import { useQuery } from "@apollo/client";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { ME_QUERY } from "@/gql/user.graphql";

export const UserMenu = () => {
	const { data: me, loading } = useQuery(ME_QUERY);

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
		<div className="flex select-none flex-row items-center gap-x-3">
			<Avatar className="h-8 w-8">
				<AvatarImage src={user.avatar} />
				<AvatarFallback>{user.name.slice(0, 2)}</AvatarFallback>
			</Avatar>
			<span className="text-sm text-gray-800">{user.name}</span>
		</div>
	);
};
