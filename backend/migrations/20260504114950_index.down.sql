alter table `file_embedding` drop key `idx_file_id`;

alter table `folder` drop key `idx_parent_name`;

alter table `file` drop key `idx_parent_name`;

alter table `folder` drop column `depth`;
alter table `folder` drop column `parent_kind`;
alter table `folder` rename column `parent_id` to `parent_folder_id`;

alter table `file` drop column `parent_kind`;
alter table `file` rename column `parent_id` to `parent_folder_id`;

alter table `file` drop column `storage_type`;

drop table `text_storage`;
