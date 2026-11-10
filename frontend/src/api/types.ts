import type { components } from "./generated/schema";

export type PrivateAccountInfo = components['schemas']['PrivateAccountInfo'];
export type AdminUserInfo = components['schemas']['AdminUserInfo'];
export type User = components['schemas']['User'];
export type UserId = components['schemas']['UserId'];
export type ServerSettings = components['schemas']['ServerSettings'];
export type LoginRequest = components['schemas']['LoginRequest'];
export type CreateAccountRequest = components['schemas']['CreateAccountRequest'];
export type CreateSingleProjectAccountRequest = components['schemas']['CreateSingleProjectAccountRequest'];
export type CreateSingleProjectAccountResponse = components['schemas']['CreateSingleProjectAccountResponse'];
export type ContactsList = components['schemas']['ContactsList'];
export type CreateProjectDetails = components['schemas']['CreateProjectDetails'];
export type ProjectId = components['schemas']['ProjectId'];
export type UserInvite = components['schemas']['UserInvite'];
export type ProjectInvite = components['schemas']['ProjectInvite'];
export type JoinProjectRequest = components['schemas']['JoinProjectRequest'];
export type ProjectPermissionLevel = components['schemas']['ProjectPermissionLevel'];
export type Compiler = components['schemas']['Compiler'];
export type ProjectMemberInfo = components['schemas']['ProjectMemberInfo'];

// Websocket
export type WSClientMessage = components['schemas']['WSClientMessage'];
export type WSServerMessage = components['schemas']['WSServerMessage'];
export type WSServerMessageType = components['schemas']['WSServerMessage']['t'];

// Only client side. Server does not know anything about the metadata structure.
export interface UserMeta {
  tags: string[]
}

// Replaces the generic UserMeta with the typed client side version from above.
export type ProjectSummary = Omit<components['schemas']['ProjectSummary'], 'metadata'> & {
  metadata: UserMeta
}
