import { z } from "zod";
import { ProjectSchema } from "@/schemas/project.schema";

const _ProjectListSchema = z.array(ProjectSchema);

// export const useProjects = (): {
// 	projects: Project[];
// 	loading: boolean;
// 	error?: Error | null;
// } => {
// 	const { data, loading } = useQuery(PROJECTS_QUERY, {});

// 	console.log(data);

// 	if (loading || !data) {
// 		return {
// 			projects: [],
// 			loading: true,
// 		};
// 	} else {
// 		const rawData = ProjectListSchema.safeParse(data?.me?.projects);

// 		if (!rawData.success) {
// 			throw new Error(rawData.error.message);
// 		} else {
// 			return {
// 				projects: rawData.data,
// 				loading: false,
// 			};
// 		}
// 	}
// };
