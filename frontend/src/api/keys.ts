export const PROFILE_KEY = "users/me";
export const DASHBOARD_KEY = "users/me/dashboard";
export const SESSIONS_KEY = "users/me/sessions?limit=32";
export const COMPILERS_KEY = "code/compilers";

export const fileContentKey = (id: string) => `files/${id}/content`;
export const projectKey = (id: string) => `projects/${id}`;
export const folderKey = (id: string) => `folders/${id}`;

export const podcastKey = (projectId: string, podcastId: string) =>
  `projects/${projectId}/podcasts/${podcastId}`;
