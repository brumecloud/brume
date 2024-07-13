import { Button } from "@/components/ui/button";
import {
  DialogHeader,
  DialogFooter,
  Dialog,
  DialogContent,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { CreateProject } from "@/gql/project.graphql";
import { ME_QUERY } from "@/gql/user.graphql";
import { ProjectSchema, type Project } from "@/schemas/project.schema";
import { projectState } from "@/state/project.state";
import { useMutation } from "@apollo/client";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { useSnapshot } from "valtio";
import { z } from "zod";

const projectSchema = z.object({
  name: z.string().min(5, {
    message: "Project name must be at least 5 characters",
  }),
  description: z.string().optional(),
});

export const CreateProjectModal = () => {
  const snap = useSnapshot(projectState);

  const [createProjectMutation, { data, error }] = useMutation(CreateProject, {
    update(cache, { data }) {
      // add the project to the me query (for the navbar)
      const meQuery = cache.readQuery({ query: ME_QUERY }) as {
        me: {
          projects: Project[];
        };
      };
      const projects: Project[] = meQuery.me.projects;

      const newProject = ProjectSchema.safeParse(data.createProject);
      if (newProject.error) {
        // this is not possible
        throw newProject.error;
      }
      const updatedProjects = [newProject.data, ...projects];
      cache.writeQuery({
        query: ME_QUERY,
        data: {
          me: {
            ...meQuery.me,
            projects: updatedProjects,
          },
        },
      });
    },
  });

  const form = useForm<z.infer<typeof projectSchema>>({
    resolver: zodResolver(projectSchema),
    defaultValues: {
      name: "",
      description: "",
    },
  });

  const createProject = (values: z.infer<typeof projectSchema>) => {
    createProjectMutation({
      variables: values,
    }).then((d) => {
      console.log(d, data, error);
      formClose();
    });
  };

  const formClose = () => {
    snap.setProjectModalOpen(false);
    form.reset();
  };

  return (
    <Form {...form}>
      <Dialog open={snap.projectModalOpen} onOpenChange={formClose}>
        <DialogContent className="bg-white shadow-sm">
          <form onSubmit={form.handleSubmit(createProject)}>
            <DialogHeader>
              <DialogTitle>New project</DialogTitle>
              <DialogDescription>Created in seconds, deployed instantly</DialogDescription>
            </DialogHeader>
            <div className="flex flex-col items-center gap-4 py-4">
              <FormField
                control={form.control}
                name="name"
                render={({ field }) => (
                  <FormItem className="w-full">
                    <FormLabel>Project</FormLabel>
                    <FormControl>
                      <Input {...field} placeholder="Project name" className="w-full" />
                    </FormControl>
                    <FormDescription>This the global name of the project</FormDescription>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <FormField
                control={form.control}
                name="description"
                render={({ field }) => (
                  <FormItem className="w-full">
                    <FormLabel>Description</FormLabel>
                    <FormControl>
                      <Textarea {...field} placeholder="Project description" className="w-full" />
                    </FormControl>
                    <FormDescription>
                      Describe your project for everybody to understand what it does
                    </FormDescription>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>
            <DialogFooter>
              <Button type="submit" variant="default">
                Do it!
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </Form>
  );
};
