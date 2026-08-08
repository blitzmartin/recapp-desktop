import { Spinner } from "@/components/shared";
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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { BackendResponse, SearchData, Timerange } from "@/types";
import { zodResolver } from "@hookform/resolvers/zod";
import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import { useForm } from "react-hook-form";
import { z } from "zod";
import { Button } from "../components/ui/button";

const searchSeriesValidationSchema = z.object({
  series_title: z
    .string({ required_error: "Title is required" })
    .min(2, { message: "Title is not valid" }),
  season_number: z.string({ required_error: "Season number is required" }),
  episode_number: z.string({ required_error: "Episode number is required" }),
  timerange: z.enum(Object.values(Timerange) as [string, ...string[]]),
});

type SearchSeriesFormValues = z.infer<typeof searchSeriesValidationSchema>;

export const SearchSeries = () => {
  const [summary, setSummary] = useState("");
  const [error, setError] = useState("");

  const searchSeriesForm = useForm<SearchSeriesFormValues>({
    defaultValues: {
      series_title: "",
      season_number: "1",
      episode_number: "1",
      timerange: Timerange.ALL_SERIES,
    },
    resolver: zodResolver(searchSeriesValidationSchema),
  });

  const onSubmit = async (values: SearchSeriesFormValues) => {
    setSummary("");
    setError("");

    const searchData: SearchData = {
      series_title: values.series_title,
      season_number: Number(values.season_number),
      episode_number: Number(values.episode_number),
      timerange: values.timerange as Timerange,
    };

    try {
      const response = await invoke<BackendResponse>("generate_recap", {
        searchData,
      });
      if (response.summary) setSummary(response.summary);
    } catch (err) {
      console.error("Error:", err);
      setError(typeof err === "string" ? err : "Something went wrong. Please try again.");
    }
  };

  return (
    <div className="flex flex-col items-center gap-6 w-full overflow-y-auto">
      <Form {...searchSeriesForm}>
        <form
          onSubmit={searchSeriesForm.handleSubmit(onSubmit)}
          className="space-y-8"
        >
          <FormField
            control={searchSeriesForm.control}
            name="series_title"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Title:</FormLabel>
                <FormControl>
                  <Input
                    className="w-full max-w-lg"
                    placeholder="Series Title"
                    {...field}
                  />
                </FormControl>
                <FormDescription>
                  Enter the title in English, as used by TMDB, even if you selected a different recap language.
                </FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />
          <FormField
            control={searchSeriesForm.control}
            name="season_number"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Season:</FormLabel>
                <FormControl>
                  <Input
                    className="w-45"
                    type="number"
                    {...field}
                    min={1}
                  />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />
          <FormField
            control={searchSeriesForm.control}
            name="episode_number"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Episode:</FormLabel>
                <FormControl>
                  <Input
                    className="w-45"
                    type="number"
                    {...field}
                    min={1}
                  />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />
          <FormField
            control={searchSeriesForm.control}
            name="timerange"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Timerange:</FormLabel>
                <Select
                  onValueChange={field.onChange}
                  defaultValue={field.value}
                >
                  <FormControl>
                    <SelectTrigger className="w-45">
                      <SelectValue placeholder="Select timerange" />
                    </SelectTrigger>
                  </FormControl>
                  <SelectContent>
                    <SelectItem value={Timerange.ALL_SERIES} defaultChecked>
                      All series until now
                    </SelectItem>
                    <SelectItem value={Timerange.ALL_SEASON}>
                      Season until now
                    </SelectItem>
                    <SelectItem value={Timerange.THIS_EPISODE}>
                      This Episode
                    </SelectItem>
                  </SelectContent>
                </Select>
                <FormMessage />
              </FormItem>
            )}
          />
          <div className="flex items-center">
            <Button
              type="submit"
              className="min-w-20 px-2"
              variant="secondary"
              disabled={searchSeriesForm.formState.isSubmitting}
            >
              Recap! {searchSeriesForm.formState.isSubmitting && <Spinner />}
            </Button>
          </div>
        </form>
      </Form>
      {error && (
        <div className="max-w-2xl p-5 text-destructive" role="alert">
          {error}
        </div>
      )}
      <div className="max-w-2xl p-5">{summary}</div>
    </div>
  );
};
