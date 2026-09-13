import { type Component } from "vue";
import type { SettingsConfiguration } from "@/components/settings/settingsSchema";
import type { ProjectSummary } from "@/services/api/managementApi";
import ConfirmationModal from "@/config/modals/ConfirmationModal.vue";

export const projectSettingsConfig = {
  defaultTabId: "general",
  tabs: [
    {
      id: "general",
      label: "General",
      icon: "tune",
      sections: [
        {
          id: "project_info",
          title: "Project Information",
          items: [
            {
              id: "project_title",
              type: "textinput",
              title: "Project Title",
              description: "The displayed name of this project.",
              inputType: "text",
              icon: "folder",
              getValue: (store) => store.context?.title ?? "",
              setValue: (value, store) => {
                if (store.context) {
                  store.context.title = value;
                }
              },
            },
            {
              id: "project_id",
              type: "text",
              title: "Project ID",
              description: "The identifier used in the URL.",
              inputType: "text",
              icon: "code",
              getValue: (store) => store.context?.id ?? "",
            },
          ],
        },
        {
          id: "editor",
          title: "Editor",
          items: [
            {
              id: "open_by_default",
              type: "select",
              title: "Open by Default",
              description: "Select a file to open automatically when loading the project.",
              defaultValue: "",
              icon: "draft",
              options: [],
            }
          ],
        },
        {
          id: "danger_zone",
          title: "Danger Zone",
          items: [
            {
              id: "delete_project",
              type: "action",
              title: "Delete Project",
              description: "Permanently removes this project and all contained files.",
              actionStyle: "button",
              actionLabel: "Delete Project",
              destructive: true,
              icon: "delete",
              modalComponent: ConfirmationModal as Component,
              modalProps: {
                title: "Delete Project?",
                message: "All project files will be permanently deleted.",
                confirmLabel: "Delete Project",
                cancelLabel: "Cancel",
                variant: "error",
              },
            },
          ],
        },
      ],
    },
  ],
} as const satisfies SettingsConfiguration<ProjectSummary>;