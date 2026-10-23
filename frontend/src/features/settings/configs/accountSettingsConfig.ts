import { type Component } from "vue";
import type { SettingsConfiguration } from "@/features/settings/utils/settingsSchema";
import ConfirmationModal from "@/shared/components/composites/ConfirmationModal.vue";
import { useAuthStore } from "@/shared/stores/authStore";
import UserManagementRow from "@/features/settings/components/custom/rows/UserManagementRow.vue";

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
            {
              id: "userId",
              type: "text",
              textStyle: "code",
              title: "User ID",
              description: "Your account identifier.",
              icon: "code",
              getValue: () => useAuthStore().state.user?.id,
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
              icon: "delete",
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
              ],
            },
          ],
        },
        {
          id: "documentViewer",
          title: "Document Viewer",
          hidden: true,
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
            {
              id: "users",
              title: "Manage Users",
              description: "View and manage all users of your Textury instance.",
              type: "page",
              icon: "manage_accounts",
              sections: [
                {
                  id: "userManagementSection",
                  items: [
                    {
                      id: "userList",
                      type: "custom",
                      title: "Custom User Management",
                      customComponent: UserManagementRow as Component,
                    }
                  ]
                }
              ]
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
              id: "maxProjectCount",
              type: "textinput",
              title: "Maximum Project Count",
              description: "Maximum number of projects any user is allowed to own before he can no longer create new projects.",
              defaultValue: 42,
              icon: "data_usage",
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
          id: "compilation",
          title: "Compilation",
          items: [
            {
              id: "maxCompileDuration",
              type: "textinput",
              title: "Maximum Compilation Duration",
              description: "Number of seconds a single compilation is allowed to take.",
              defaultValue: 42,
              icon: "timer",
            },
            {
              id: "maxCompileMemory",
              type: "textinput",
              title: "Maximum Compilation Memory",
              description: "Number of kilobytes a single compilation is allowed to take.",
              defaultValue: 42,
              icon: "memory_alt",
            },
          ],
        },
      ],
    },
    {
      id: "about",
      label: "About",
      icon: "info",
      sections: [
        {
          id: "aboutInformation",
          title: "About",
          items: [
            {
              id: "version",
              type: "text",
              title: "Version",
              description: "Current running server and client version.",
              icon: "tag",
              getValue: () => {
                const version = import.meta.env.PACKAGE_VERSION;
                return "v" + version;
              }
            },
            {
              id: "repository",
              type: "action",
              title: "Repository",
              description: "Check out the source code on Codeberg.",
              icon: "code",
              onClick: () => {
                const newWindow = window.open('https://codeberg.org/derdilla/textury', '_blank', 'noopener,noreferrer');
                if (newWindow) {
                  newWindow.focus();
                }
              }
            }
          ]
        }
      ]
    }
  ],
} as const satisfies SettingsConfiguration;
