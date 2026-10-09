import { type Component } from "vue";
import type { SettingsConfiguration } from "@/features/settings/utils/settingsSchema";
import ConfirmationModal from "@/shared/components/composites/ConfirmationModal.vue";
import { useAuthStore } from "@/shared/stores/authStore";

export const accountSettingsConfig = {
  defaultTabId: "profile",
  tabs: [
    {
      id: "profile",
      label: "Profile",
      icon: "person",
      sections: [
        {
          id: "identity",
          title: "Identity",
          items: [
            {
              id: "username",
              type: "textinput",
              title: "Username",
              description: "Publicly displayed name for your account.",
              inputType: "text",
              icon: "badge",
              noStoreValue: true,
              getValue: () => useAuthStore().state.user?.username,
              setValue: async (value) => {
                const res = await useAuthStore().updateUsername(value);
                if (!res.success) {
                  console.warn("Failed to update username");
                }
              },
            },
            {
              id: "email",
              type: "textinput",
              title: "Email",
              description: "Email address used for sign-in.",
              icon: "mail",
              noStoreValue: true,
              getValue: () => useAuthStore().state.user?.email,
              setValue: async (value) => {
                const res = await useAuthStore().updateEmail(value);
                if (!res.success) {
                  console.warn("Failed to update email");
                }
              },
            },
          ],
        },
        {
          id: "dangerZone",
          title: "Danger Zone",
          items: [
            {
              id: "deleteAccount",
              type: "action",
              title: "Delete Account",
              description: "Permanently removes your account and all associated data.",
              actionStyle: "button",
              actionLabel: "Delete Account",
              destructive: true,
              icon: "delete_forever",
              modalComponent: ConfirmationModal as Component,
              modalProps: {
                title: "Delete Account?",
                message: "All data, settings, and access permissions will be permanently deleted.",
                confirmLabel: "Delete Account",
                cancelLabel: "Cancel",
                variant: "error",
                onConfirm: () => useAuthStore().deleteAccount(),
              },
            },
          ],
        },
      ],
    },
    {
      id: "appearance",
      label: "Appearance",
      icon: "palette",
      sections: [
        {
          id: "theme",
          title: "Theme",
          items: [
            {
              id: "colorScheme",
              type: "select",
              title: "Color Scheme",
              description: "Select your preferred application theme.",
              defaultValue: "auto",
              icon: "dark_mode",
              options: [
                { label: "Automatic", value: "auto" },
                { label: "Light", value: "light" },
                { label: "Dark", value: "dark" },
                { label: "Black", value: "black" },
              ],
            },
          ],
        },
        {
          id: "documentViewer",
          title: "Document Viewer",
          items: [
            {
              id: "pdfViewerEngine",
              type: "select",
              title: "PDF Viewer Engine",
              description: "Determines how PDF documents are rendered.",
              defaultValue: "native",
              icon: "picture_as_pdf",
              options: [
                { label: "Native Viewer", value: "native" },
                { label: "Embedded Viewer", value: "embedded" },
              ],
            },
          ],
        },
      ],
    },
    {
      id: "admin",
      label: "Administration",
      icon: "admin_panel_settings",
      hidden: () => !useAuthStore().state.user?.isAdmin,
      sections: [
        {
          id: "userManagement",
          title: "User Management",
          items: [
            {
              id: "openSignUps",
              type: "switch",
              title: "Allow Open Signups",
              description: "Allows new users to register without an invitation.",
              defaultValue: false,
              icon: "how_to_reg",
            },
          ],
        },
        {
          id: "policies",
          title: "Policies",
          items: [
            {
              id: "userProjects",
              type: "switch",
              title: "Allow Creating Projects",
              description: "Permits regular users to create new projects.",
              defaultValue: true,
              icon: "add_box",
            },
            {
              id: "openSharing",
              type: "switch",
              title: "Allow External Sharing",
              description: "Enables sharing projects with users outside the organization.",
              defaultValue: false,
              icon: "share",
            },
          ],
        },
        {
          id: "sandboxes",
          title: "Sandboxes",
          items: [
            {
              id: "resetSandboxes",
              type: "action",
              title: "Reset Sandboxes",
              description: "Resets all active sandbox environments to their initial state.",
              actionStyle: "button",
              actionLabel: "Reset Sandboxes",
              destructive: true,
              icon: "restart_alt",
              modalComponent: ConfirmationModal as Component,
              modalProps: {
                title: "Reset All Sandboxes?",
                message: "No project data will be lost. Only temporary files will be deleted.",
                confirmLabel: "Reset Sandboxes",
                cancelLabel: "Cancel",
              },
            },
          ],
        },
      ],
    },
  ],
} as const satisfies SettingsConfiguration;
