``
async function createFile(fileName: !string, initialContent: object) {
  const response = await fetch(`${API_ROOT}/project/${projectId}`, {
    method: 'PUT',
    body: initialContent,
  });
  return {
    success: response.ok,
    message: response.statusText,
  };
}
