import { API_ROOT } from '@/services/managementApi';
import type { ProjectSummary } from '@/services/managementApi';

export class ProjectApi {
  projectId!: string;
  constructor(projectId: string) {
    this.projectId = projectId;
  }

  async index(): Promise<ProjectIndex | null> {
    const response = await fetch(`${API_ROOT}/project/${this.projectId}/index`);
    try {
      return await response.json();
    } catch (e) {
      console.error(e);
      return null;
    }
  }

  ySocketUrl(method = "edit") {
    const protocol = location.protocol == 'https:' ? 'wss:' : 'ws:';
    return `${protocol}//${location.hostname}:${location.port}${API_ROOT}/project/${method}/${this.projectId}`;
  }
}

export interface ProjectIndex {
  summary: ProjectSummary,
}

export interface FileMetadata {
  whole_path: string,
  name: string,
  file_type: FileType,
}

export type FileType =  {
  "Text"?: {
    size: number,
  },
  "Binary"?: {
    size: number,
  },
  "Directory"?: {
    children: FileMetadata[],
  }
};
