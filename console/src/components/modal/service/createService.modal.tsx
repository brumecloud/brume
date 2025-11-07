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
import { useAddService } from "@/hooks/useAddService";
import { useProject } from "@/hooks/useProject";
import { modalState } from "@/state/modal.state";
import { zodResolver } from "@hookform/resolvers/zod";
import { Container } from "lucide-react";
import { useForm } from "react-hook-form";
import { useSnapshot } from "valtio";
import { z } from "zod";

const serviceSchema = z.object({
  name: z.string().min(5, {
    message: "Service name must be at least 5 characters",
  }),
  image: z.string(),
});

export const CreateServiceModal = () => {
  const snap = useSnapshot(modalState);
  const { project } = useProject();
  const { addServiceMutation, loading } = useAddService(project?.id || "");

  const form = useForm<z.infer<typeof serviceSchema>>({
    resolver: zodResolver(serviceSchema),
    defaultValues: {
      name: "",
      image: "",
    },
  });

  const createService = (values: z.infer<typeof serviceSchema>) => {
    addServiceMutation({
      variables: {
        input: {
          name: values.name,
          image: values.image,
        },
        projectId: project?.id,
      },
    }).then(() => {
      formClose();
    });
  };

  const formClose = () => {
    snap.setCreateServiceModalOpen(false);
    form.reset();
  };

  return (
    <Form {...form}>
      <Dialog open={snap.createServiceModalOpen} onOpenChange={formClose}>
        <DialogContent className="bg-white shadow-sm">
          <form onSubmit={form.handleSubmit(createService)}>
            <DialogHeader>
              <DialogTitle>Add a new service</DialogTitle>
              <DialogDescription>
                Your project would look very empty without some services
              </DialogDescription>
            </DialogHeader>
            <div className="flex flex-col gap-4 py-4">
              <FormField
                control={form.control}
                name="name"
                disabled={loading}
                render={({ field }) => (
                  <FormItem className="w-full">
                    <FormLabel>Service</FormLabel>
                    <FormControl>
                      <Input
                        {...field}
                        placeholder="Service name"
                        className="w-full"
                      />
                    </FormControl>
                    <FormDescription>
                      This is the name of the service
                    </FormDescription>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <hr />
              <div className="flex items-center gap-x-2">
                <Container className="w-5 text-slate-800" />
                <p className="font-semibold text-slate-800 text-sm">
                  Docker Executor
                </p>
              </div>
              <p className="font-normal text-slate-500 text-sm">
                Run any docker image (from docker.io) registry at the moment
              </p>
              <FormField
                control={form.control}
                name="image"
                disabled={loading}
                render={({ field }) => (
                  <FormItem className="w-full">
                    <FormLabel>Image</FormLabel>
                    <FormControl>
                      <Input
                        {...field}
                        placeholder="image"
                        className="w-full font-mono"
                      />
                    </FormControl>
                    <FormDescription>
                      This is the name of the docker.io image we will run
                    </FormDescription>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>
            <DialogFooter>
              <Button disabled={loading} type="submit" variant="default">
                Add it!
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </Form>
  );
};
