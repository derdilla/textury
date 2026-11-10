import { api } from "@/api";
import { useEditorFileTree } from "@/features/editor/composables/editorFileTree";
import { useEditorWS, type EditorWS, ConnectionState } from "@/features/editor/composables/editorWS";
import { useProjectPublicShareStore } from "@/features/projects/stores/projectPublicShareStore";
import { useAuthStore } from "@/shared/stores/authStore";
import { useProjectsStore } from "@/shared/stores/projectsStore";
import { expect, test } from 'vitest';

test('can manage project members via settings', async () => {
  const initialToken = await (await fetch('/testGetInitialRegistrationToken')).json();

  const authStore = useAuthStore();
  const result = await authStore.register({
    username: `testadmin+${initialToken}@admin.de`,
    email: `testadmin+${initialToken}@admin.de`,
    password: `adminadmin`,
    registrationToken: initialToken,
  });
  expect(result.success).toBeTruthy();

  const projectsStore = useProjectsStore();
  await projectsStore.createProject('Test project', 'none');
  await projectsStore.fetchProjects();
  const allProjects = projectsStore.projects.value;
  expect(allProjects.length).toBe(1);

  const shareToken = await projectsStore.createShareToken(allProjects[0]!.id, "compile");
  expect(shareToken.success).toBeTruthy();

  const inviteToken = await api.POST("/settings/users/invite");
  expect(inviteToken.data).toBeDefined();

  // register member
  await authStore.logout();
  await authStore.register({
    username: `testauser${inviteToken.data!.registrationToken}@admin.de`,
    email: `testauser${inviteToken.data!.registrationToken}@admin.de`,
    password: `adminuser`,
    registrationToken: inviteToken.data!.registrationToken,
  });
  const invitedUserId = authStore.state.user?.id;

  // Check member has correct permissions on join
  await projectsStore.joinProject({ token: shareToken.data!.token });
  await projectsStore.fetchProjects();
  expect(projectsStore.projects.value).toHaveLength(1);
  expect(projectsStore.projects.value[0]!.permissions).toEqual("compile");


  // go back to admin user and demote user
  await authStore.logout();
  await authStore.login({
    email: `testadmin+${initialToken}@admin.de`,
    password: `adminadmin`,
  });
  const userPermissions = await api.GET("/project/{id}/members", {
    params: { "path": {"id": projectsStore.projects.value[0]!.id }}
  });
  expect(userPermissions.data).toHaveLength(2);
  expect(userPermissions.data).toContainEqual({
    "id": invitedUserId,
    "permissions": "compile",
  });
  expect(userPermissions.data).toContainEqual({
    "id": authStore.state.user?.id,
    "permissions": "edit",
  });
  await api.PUT("/project/{pid}/members/{uid}/permissions", {
    params: {
      path: {
        "pid": projectsStore.projects.value[0]!.id,
        "uid": invitedUserId!,
      },
    },
    body: "view",
  });

  // go back to user and check demotion works
  await authStore.logout();
  await authStore.login({
    email: `testauser${inviteToken.data!.registrationToken}@admin.de`,
    password: `adminuser`,
  });
  await projectsStore.fetchProjects();
  expect(projectsStore.projects.value).toHaveLength(1);
  expect(projectsStore.projects.value[0]!.permissions).toBe("view");
});

test('can manage project members via settings', async () => {
  const initialToken = await (await fetch('/testGetInitialRegistrationToken')).json();

  const authStore = useAuthStore();
  const result = await authStore.register({
    username: `testadmin+${initialToken}@admin.de`,
    email: `testadmin+${initialToken}@admin.de`,
    password: `adminadmin`,
    registrationToken: initialToken,
  });
  const adminUserId = authStore.state.user?.id;
  expect(result.success).toBeTruthy();

  const projectsStore = useProjectsStore();
  await projectsStore.createProject('Test project', 'none');
  await projectsStore.fetchProjects();

  await expect(api.PUT('/settings/openSharing', { body: true })).resolves.toSatisfy(val => val.response.ok);
  const publicSharesStore = useProjectPublicShareStore(projectsStore.projects.value[0]!.id);
  expect(publicSharesStore.state.value.isShared).toBeFalsy();
  const shareId = `share-test-${projectsStore.projects.value[0]!.id}`;
  expect(publicSharesStore.setId(shareId)).toBeNull();
  expect(publicSharesStore.setPassword(`${shareId}-pw`)).toBeNull();
  expect(publicSharesStore.setInitialPermission('view')).toBeNull();
  await expect(publicSharesStore.share()).resolves.toBeNull();


  // register member
  await authStore.logout();
  const joinResult = await authStore.joinProject({ shareId: shareId, password: `${shareId}-pw` });
  expect(joinResult.success).toBeTruthy();
  const invitedUserId = authStore.state.user?.id;
  expect(invitedUserId).not.toEqual(adminUserId);
  await projectsStore.fetchProjects();
  expect(projectsStore.projects.value).toHaveLength(1);
  expect(projectsStore.projects.value[0]!.permissions).toEqual("view");
});

test('invite tokens are different', async () => {
  const initialToken = await (await fetch('/testGetInitialRegistrationToken')).json();

  const authStore = useAuthStore();
  const result = await authStore.register({
    username: `testadmin+${initialToken}@admin.de`,
    email: `testadmin+${initialToken}@admin.de`,
    password: `adminadmin`,
    registrationToken: initialToken,
  });
  expect(result.success).toBeTruthy();

  const projectsStore = useProjectsStore();
  await projectsStore.createProject('Test project', 'none');
  await projectsStore.fetchProjects();
  const allProjects = projectsStore.projects.value;
  expect(allProjects.length).toBe(1);

  const shareToken1 = await projectsStore.createShareToken(allProjects[0]!.id, "compile");
  const shareToken2 = await projectsStore.createShareToken(allProjects[0]!.id, "compile");
  expect(shareToken1.data).not.toEqual(shareToken2.data);

  const inviteToken1 = await api.POST("/settings/users/invite");
  const inviteToken2 = await api.POST("/settings/users/invite");
  expect(inviteToken1.data).not.toEqual(inviteToken2.data);
});

test('can transfer ownership', async () => {
  const initialToken = await (await fetch('/testGetInitialRegistrationToken')).json();

  const authStore = useAuthStore();
  const result = await authStore.register({
    username: `testadmin+${initialToken}@admin.de`,
    email: `testadmin+${initialToken}@admin.de`,
    password: `adminadmin`,
    registrationToken: initialToken,
  });
  const adminUserId = authStore.state.user?.id;
  expect(result.success).toBeTruthy();

  const projectsStore = useProjectsStore();
  await projectsStore.createProject('Test project', 'none');
  await projectsStore.fetchProjects();
  const allProjects = projectsStore.projects.value;
  expect(allProjects.length).toBe(1);

  const shareToken1 = await projectsStore.createShareToken(allProjects[0]!.id, "compile");
  const inviteToken1 = await api.POST("/settings/users/invite");

  // register member
  await authStore.logout();
  await authStore.register({
    username: `testauser${inviteToken1.data!.registrationToken}@admin.de`,
    email: `testauser${inviteToken1.data!.registrationToken}@admin.de`,
    password: `adminuser`,
    registrationToken: inviteToken1.data!.registrationToken,
  });
  const invitedUserId = authStore.state.user?.id;
  await projectsStore.joinProject({ token: shareToken1.data!.token });
  await projectsStore.fetchProjects();
  expect(projectsStore.projects.value[0]!.owner).toBe(adminUserId);

  // self-transfer impossible
  const resNonAuth = await api.POST("/project/{pid}/members/{uid}/transferOwnership", {
    params: {
      path: {
        "pid": projectsStore.projects.value[0]!.id,
        "uid": invitedUserId!,
      },
    },
  });
  expect(resNonAuth.response.ok).toBeFalsy();

  // Go back to admin user
  await authStore.logout();
  await authStore.login({
    email: `testadmin+${initialToken}@admin.de`,
    password: `adminadmin`,
  });
  await projectsStore.fetchProjects();
  expect(projectsStore.projects.value[0]!.owner).toBe(adminUserId);

  const resAuth = await api.POST("/project/{pid}/members/{uid}/transferOwnership", {
    params: {
      path: {
        "pid": projectsStore.projects.value[0]!.id,
        "uid": invitedUserId!,
      },
    },
  });
  expect(resAuth.response.ok).toBeTruthy();
  await projectsStore.fetchProjects();
  expect(projectsStore.projects.value[0]!.owner).toBe(invitedUserId);
});
