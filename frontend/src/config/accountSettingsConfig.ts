import { type Component } from "vue";
import type { SettingsConfiguration } from "@/components/settings/settingsSchema";
import ConfirmationModal from "@/config/modals/ConfirmationModal.vue";
import { useAuthStore } from "@/stores/auth";

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
              getValue: () => useAuthStore().state.user?.username,
              setValue: (value) => {
                const auth = useAuthStore();
                if (auth.state.user) {
                  auth.state.user.username = value;
                }
              },
            },
            {
              id: "email",
              type: "textinput",
              title: "Email",
              description: "Email address used for sign-in.",
              icon: "mail",
              getValue: () => useAuthStore().state.user?.email,
              setValue: (value) => {
                const auth = useAuthStore();
                if (auth.state.user) {
                  auth.state.user.email = value;
                }
              },
            },
          ],
        },
        {
          id: "danger_zone",
          title: "Danger Zone",
          items: [
            {
              id: "delete_account",
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
              id: "color_scheme",
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
          id: "document_viewer",
          title: "Document Viewer",
          items: [
            {
              id: "pdf_viewer_engine",
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
          id: "user_management",
          title: "User Management",
          items: [
            {
              id: "allow_open_signups",
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
              id: "allow_creating_projects",
              type: "switch",
              title: "Allow Creating Projects",
              description: "Permits regular users to create new projects.",
              defaultValue: true,
              icon: "add_box",
            },
            {
              id: "allow_sharing_projects_external",
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
              id: "reset_sandboxes",
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