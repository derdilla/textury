import { type Component } from "vue";
import type { SettingsConfiguration } from "@/features/settings/utils/settingsSchema";
import ConfirmationModal from "@/shared/components/composites/ConfirmationModal.vue";
import { useProjectsStore } from "@/shared/stores/projectsStore";
import ShareProjectPubliclyForm from "@/features/projects/components/ShareProjectPubliclyForm.vue";
import { useProjectPublicShareStore } from "@/features/projects/stores/projectPublicShareStore";
import { useAccountSettingsStore } from "../stores/accountSettingsStore";

export const projectSettingsConfig = {
  defaultTabId: "general",
  tabs: [
    {
      id: "general",
      label: "General",
      icon: "tune",
      sections: [
        {
          id: "projectInfo",
          title: "Project Information",
          items: [
            {
              id: "projectTitle",
              type: "textinput",
              title: "Project Title",
              description: "The displayed name of this project.",
              inputType: "text",
              icon: "folder",
              getValue: (instance) => {
                if (!instance.context) return undefined;
                const project = useProjectsStore().getProjectById(instance.context);
                return project?.title;
              },
              setValue: async (value, instance) => {
                if (!instance.context) return;
                const projectsStore = useProjectsStore();
                const project = projectsStore.getProjectById(instance.context);
                if (project) {
                  project.title = value;
                  const res = await useProjectsStore().updateTitle(instance.context, value);
                  if (!res.success) {
                    console.warn("Failed to update project title");
                  }
                }
              }
            },
            {
              id: "projectId",
              type: "text",
              title: "Project ID",
              description: "The identifier used in the URL.",
              inputType: "text",
              icon: "code",
              getValue: (instance) => {
                if (!instance.context) return undefined;
                const project = useProjectsStore().getProjectById(instance.context);
                return project?.id;
              },
            },
          ],
        },
        {
          id: "editor",
          title: "Editor",
          hidden: true,
          items: [
            {
              id: "openByDefault",
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
          id: "dangerZone",
          title: "Danger Zone",
          items: [
            {
              id: "deleteProject",
              type: "action",
              title: "Delete Project",
              description: "Permanently removes this project and all contained files.",
              actionStyle: "button",
              actionLabel: "Delete Project",
              destructive: true,
              icon: "delete",
              modalComponent: ConfirmationModal as Component,
              modalProps: (instance) => ({
                title: "Delete Project?",
                message: "All project files will be permanently deleted.",
                confirmLabel: "Delete Project",
                cancelLabel: "Cancel",
                variant: "error",
                onConfirm: () => {
                  if (instance.context) {
                    useProjectsStore().deleteProject(instance.context);
                  }
                }
              }),
            },
          ],
        },
      ],
    },
    {
      id: "share",
      label: "Invites & Members",
      icon: "share",
      sections: [
        {
          id: "publicShares",
          title: "Public Share",
          items: [
            {
              id: "publicSharesDisabled",
              title: "Public sharing is disabled",
              description: "This setting was disabled by the instance admin",
              type: "text",
              icon: "info",
              important: true,
              hidden: (_) => {
                const accountSettingsStore = useAccountSettingsStore();
                if (accountSettingsStore.state.serverSettings?.openSharing) return true;
                return false;
              },
            },
            {
              id: "test_custom_with_props",
              type: "custom",
              title: "Custom Vue Row Component",
              customComponent: ShareProjectPubliclyForm as Component,
              hidden: (instance) => instance.context == null,
              customProps: (instance) => {
                return {
                  projectId: instance.context,
                };
              },
            },
            {
              id: "openSignUpCount",
              type: "text",
              title: "User Count",
              description: "Amount of times this login was used",
              hidden: (instance) => {
                if (!instance.context) return true;
                return useProjectPublicShareStore(instance.context).state.value.signUpCount === null;
              },
              getValue: (instance) => {
                if (!instance.context) return null;
                return useProjectPublicShareStore(instance.context).state.value.signUpCount;
              }
            }
          ]
        }
      ],
    },
  ],
} as const satisfies SettingsConfiguration<string>; // Context is a project id aka string.
