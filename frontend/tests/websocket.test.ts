import { useEditorFileTree } from "@/features/editor/composables/editorFileTree";
import { useEditorWS, type EditorWS, ConnectionState } from "@/features/editor/composables/editorWS";
import { useAuthStore } from "@/shared/stores/authStore";
import { useProjectsStore } from "@/shared/stores/projectsStore";
import { expect, test } from 'vitest';

test('connects to project via websocket', async () => {
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
  const res = await projectsStore.createProject('Test project', 'none');
  expect(res.error).toBeNullable();
  await projectsStore.fetchProjects();
  const allProjects = projectsStore.projects.value;
  expect(allProjects.length).toBe(1);

  const ws: EditorWS = useEditorWS(allProjects[0]!.id);
  ws.connect();
  expect(ws.state.value).toBe(ConnectionState.EstablishingConnection);
  await expect.poll(() => ws.state.value, {
    timeout: 500,
    interval: 20,
  }).toBe(ConnectionState.Connected);
});

test('disconnects on account deletion', async () => {
  const initialToken = await (await fetch('/testGetInitialRegistrationToken')).json();

  const authStore = useAuthStore();
  await authStore.register({
    username: `testadmin+${initialToken}@admin.de`,
    email: `testadmin+${initialToken}@admin.de`,
    password: `adminadmin`,
    registrationToken: initialToken,
  });

  const projectsStore = useProjectsStore();
  await projectsStore.createProject('Test project', 'none');
  await projectsStore.fetchProjects();
  const allProjects = projectsStore.projects.value;
  expect(allProjects.length).toBe(1);

  const ws: EditorWS = useEditorWS(allProjects[0]!.id);
  ws.connect();
  expect(ws.state.value).toBe(ConnectionState.EstablishingConnection);
  await expect.poll(() => ws.state.value, {
    timeout: 500,
    interval: 20,
  }).toBe(ConnectionState.Connected);

  await expect(authStore.deleteAccount()).resolves.toSatisfy((r) => r.success);
  await expect.poll(() => ws.state.value, {
    timeout: 500,
    interval: 20,
  }).toBe(ConnectionState.Disconnected);
});

test('Can compile documents', async () => {
  const initialToken = await (await fetch('/testGetInitialRegistrationToken')).json();

  const authStore = useAuthStore();
  await authStore.register({
    username: `testadmin+${initialToken}@admin.de`,
    email: `testadmin+${initialToken}@admin.de`,
    password: `adminadmin`,
    registrationToken: initialToken,
  });

  const projectsStore = useProjectsStore();
  await projectsStore.createProject('Test project', 'none');
  await projectsStore.fetchProjects();
  const allProjects = projectsStore.projects.value;

  const ws: EditorWS = useEditorWS(allProjects[0]!.id);
  ws.connect();
  expect(ws.state.value).toBe(ConnectionState.EstablishingConnection);
  await expect.poll(() => ws.state.value, {
    timeout: 500,
    interval: 20,
  }).toBe(ConnectionState.Connected);

  const fileTree = useEditorFileTree(allProjects[0]!.id);
  expect(fileTree.ws).toEqual(ws);
  expect(fileTree.root.value).toBeDefined();
  const createFileRes = fileTree.root.value!.createTextFile('main.typ', '= Test Document\n#lorem(300)');
  expect(createFileRes).toEqual('ok');
  const fileUUID = fileTree.root.value?.children[0]?.uuid;
  expect(fileUUID).toBeDefined();
  expect(fileUUID!.length).toBeGreaterThan(0);

  const onResetLog = new Promise((r) => ws.on('resetLog', r));
  const onCompileResult = new Promise((r) => ws.on('setPdf', r));

  ws.send({
    't': 'compile',
    'uuid': fileUUID!,
    'compiler': 'Typst'
  });
  await expect(onResetLog).resolves.toBeDefined();
  await expect(onCompileResult).resolves.toSatisfy(res => res.uuid == fileUUID && res.content != null);
});
