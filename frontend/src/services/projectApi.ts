import { API_ROOT } from '@/services/managementApi';
import type { Compiler } from './editorController';

export class ProjectApi {
  projectId!: string;
  constructor(projectId: string) {
    this.projectId = projectId;
  }

  async index(): Promise<FileMetadata | null> {
    const response = await fetch(`${API_ROOT}/project/${this.projectId}/index`);
    try {
      const index = await response.json();
      if ((typeof index.name) !== "string") throw "Bad property name";
      if ((typeof index.file_type) !== "object") throw "Bad property file_type";
      return index;
    } catch (e) {
      console.error(e);
      return null;
    }
  }

  async createOrUpdateFile(filePath: string, initialContent: BodyInit | null | undefined) {
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
    const response = await fetch(`${API_ROOT}/project/${this.projectId}/file/${filePath}`, {
      method: 'DELETE'
    });
    return {
      success: response.ok,
      message: response.statusText,
    };
  }

  async getFile(filePath: string) {
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


export interface FileMetadata {
  whole_path: string,
  name: string,
  file_type: FileTypeText | FileTypeBinary | FileTypeDirectory,
}

export interface FileTypeText {
  "Text": {
    size: number,
  }
}

export interface FileTypeBinary {
  "Binary": {
    size: number,
  }
}

export interface FileTypeDirectory {
  "Directory": {
    children: [FileMetadata],
  }
}

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
