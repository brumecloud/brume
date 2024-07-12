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
import { projectState } from "@/state/project.state";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { useSnapshot } from "valtio";
import { z } from "zod";

const projectSchema = z.object({
  name: z.string().min(5, {
    message: "Project name must be at least 5 characters",
  }),
});

export const CreateProjectModal = () => {
  const snap = useSnapshot(projectState);

  const form = useForm<z.infer<typeof projectSchema>>({
    resolver: zodResolver(projectSchema),
    defaultValues: {
      name: "",
    },
  });

  const createProject = (values: z.infer<typeof projectSchema>) => {
    console.log(values);
    formClose();
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
            <div className="flex items-center space-x-2 py-4">
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
