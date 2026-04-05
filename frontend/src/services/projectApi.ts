import { API_ROOT } from '@/services/managementApi';
import type { Compiler } from './editorController';
import type { ProjectSummary } from '@/services/managementApi';

export class ProjectApi {
  projectId!: string;
  constructor(projectId: string) {
    this.projectId = projectId;
  }

  async index(): Promise<ProjectIndex | null> {
    const response = await fetch(`${API_ROOT}/project/${this.projectId}/index`);
    try {
      const index: ProjectIndex | undefined = await response.json();
      if ((typeof index?.root.name) !== "string") throw "Bad property name";
      if ((typeof index?.root.file_type) !== "object") throw "Bad property file_type";
      return index;
    } catch (e) {
      console.error(e);
      return null;
    }
  }

  async createOrUpdateFile(filePath: string, initialContent: BodyInit | null) {
    if (filePath.startsWith('/')) filePath = filePath.substring(1);
    const response = await fetch(`${API_ROOT}/project/${this.projectId}/file/${filePath}`, {
      method: 'PUT',
      body: initialContent,
    });
    return {
      success: response.ok,
      message: response.statusText,
    };
  }

  async deleteFile(filePath: string) {
    if (filePath.startsWith('/')) filePath = filePath.substring(1);
    const response = await fetch(`${API_ROOT}/project/${this.projectId}/file/${filePath}`, {
      method: 'DELETE'
    });
    return {
      success: response.ok,
      message: response.statusText,
    };
  }

  async moveFile(sourcePath: string, destinationPath: string) {
    if (sourcePath.startsWith('/')) sourcePath = sourcePath.substring(1);
    if (destinationPath.startsWith('/')) destinationPath = destinationPath.substring(1);
    const response = await fetch(`${API_ROOT}/project/${this.projectId}/move/${sourcePath}`, {
      method: 'POST',
      body: destinationPath,
    });
    return {
      success: response.ok,
      message: response.statusText,
    };
  }

  async getFile(filePath: string) {
    if (filePath.startsWith('/')) filePath = filePath.substring(1);
    const response = await fetch(`${API_ROOT}/project/${this.projectId}/file/${filePath}`);
    return {
      success: response.ok,
      message: response.statusText,
      content: await response.blob(),
    };
  }

  async compile(filePath: string, options: CompileOptions): Promise<CompilationResult> {
    const response = await fetch(`${API_ROOT}/project/${this.projectId}/compile${filePath}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(options),
    });
    return {
      success: response.ok,
      output: response.ok ? await response.json() : null,
    };
  }

  ySocketUrl() {
    const protocol = location.protocol == 'https:' ? 'wss:' : 'ws:';
    return `${protocol}//${location.hostname}:${location.port}${API_ROOT}/project/${this.projectId}/edit`;
  }
}

export interface ProjectIndex {
  summary: ProjectSummary,
  root: FileMetadata,
}

export interface FileMetadata {
  whole_path: string,
  name: string,
  file_type: FileTypeText | FileTypeBinary | FileTypeDirectory,
}

export type FileTypeText = {
  "Text": {
    size: number,
  }
};

export type FileTypeBinary = {
  "Binary": {
    size: number,
  }
};

export type FileTypeDirectory = {
  "Directory": {
    children: FileMetadata[],
  }
};

export interface CompileOptions {
  compiler: Compiler | null,
}

export interface CompilationResult {
  success: boolean,
  output: null | {
    log: string,
    document: string | null
  }
}
