import { kyInstance } from "./mutator";
import {
  CreateFileBodyParentKind,
  type CreateFile200,
  type CreateFileBody,
  type RequestUploadUrl200,
  type RequestUploadUrlBody,
} from "./generated/backend.schemas";

interface UploadOptions {
  file: File;
  parentId: string;
  parentKind: "project" | "folder";
  description?: string;
}

const MAX_UPLOAD_BYTES = 16 * 1024 * 1024;

export async function uploadFile(opts: UploadOptions): Promise<CreateFile200> {
  const { file, parentId, parentKind, description = "" } = opts;
  if (file.size === 0) {
    throw new Error("file_empty");
  }
  if (file.size > MAX_UPLOAD_BYTES) {
    throw new Error("file_too_large");
  }

  const reqBody: RequestUploadUrlBody = {
    content_length: file.size,
    content_type: file.type || "application/octet-stream",
  };

  const presigned = await kyInstance
    .post("files/upload-url", { json: reqBody })
    .json<RequestUploadUrl200>();

  const putRes = await fetch(presigned.url, {
    method: presigned.method,
    headers: { "Content-Type": presigned.content_type },
    body: file,
  });
  if (!putRes.ok) {
    throw new Error(`upload_failed_${putRes.status}`);
  }

  const createBody: CreateFileBody = {
    name: file.name,
    description,
    parent_id: parentId,
    parent_kind:
      parentKind === "project"
        ? CreateFileBodyParentKind.project
        : CreateFileBodyParentKind.folder,
    storage_id: presigned.storage_id,
  };

  return kyInstance
    .post("files", { json: createBody })
    .json<CreateFile200>();
}

interface CreateTextFileOptions {
  name: string;
  text: string;
  parentId: string;
  parentKind: "project" | "folder";
  description?: string;
}

const MAX_TEXT_BYTES = 16 * 1024 * 1024;

export async function createTextFile(
  opts: CreateTextFileOptions,
): Promise<CreateFile200> {
  const { name, text, parentId, parentKind, description = "" } = opts;
  if (new Blob([text]).size > MAX_TEXT_BYTES) {
    throw new Error("file_too_large");
  }
  const body: CreateFileBody = {
    name,
    description,
    parent_id: parentId,
    parent_kind:
      parentKind === "project"
        ? CreateFileBodyParentKind.project
        : CreateFileBodyParentKind.folder,
    text,
  };
  return kyInstance.post("files", { json: body }).json<CreateFile200>();
}
