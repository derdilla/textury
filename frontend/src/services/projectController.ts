import { reactive } from "vue";
import type { ProjectSummary } from "@/services/api/managementApi";

export interface ProjectController {
  sorting: Sorting;
  view: View;
  selectedTags: string[];
  projects: ProjectSummary[];
}

export enum Sorting {
  name = "name",
  dateEdited = "date-edited",
  dateCreated = "date-created",
}

export enum View {
  grid = "grid",
  list = "list",
}

export const projectCtrl: ProjectController = reactive({
  sorting: Sorting.name,
  view: View.list,
  selectedTags: [],
  projects: [],
});