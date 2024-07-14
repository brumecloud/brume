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
import { useCreateProject } from "@/hooks/useCreateProject";
import { modalState } from "@/state/modal.state";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { useSnapshot } from "valtio";
import { z } from "zod";

const serviceSchema = z.object({
  name: z.string().min(5, {
    message: "Service name must be at least 5 characters",
  }),
});

export const CreateServiceModal = () => {
  const snap = useSnapshot(modalState);
  const { createProjectMutation, loading } = useCreateProject();

  const form = useForm<z.infer<typeof serviceSchema>>({
    resolver: zodResolver(serviceSchema),
    defaultValues: {
      name: "",
    },
  });

  const createService = (values: z.infer<typeof serviceSchema>) => {
    createProjectMutation({
      variables: values,
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
              <DialogDescription>Your project would look very empty without some services</DialogDescription>
            </DialogHeader>
            <div className="flex flex-col items-center gap-4 py-4">
              <FormField
                control={form.control}
                name="name"
                disabled={loading}
                render={({ field }) => (
                  <FormItem className="w-full">
                    <FormLabel>Service</FormLabel>
                    <FormControl>
                      <Input {...field} placeholder="Service name" className="w-full" />
                    </FormControl>
                    <FormDescription>This is the name of the service</FormDescription>
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
