import { reactive } from "vue";

export interface ProjectController {
  sorting: Sorting;
  view: View;
  selectedTags: string[];
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
});