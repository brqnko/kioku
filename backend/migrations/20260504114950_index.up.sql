create table `text_storage` (
    `storage_id` varbinary(16) not null,
    `content`    longtext not null,

    `created_at` datetime not null default(now()),
    `updated_at` datetime not null default(now()) on update now(),

    primary key (`storage_id`) /*T![clustered_index] CLUSTERED */
);

alter table `file` add column `storage_type` tinyint unsigned not null default 0;
alter table `file` alter column `storage_type` drop default;

alter table `file` rename column `parent_folder_id` to `parent_id`;
alter table `file` add column `parent_kind` tinyint unsigned not null default 0;
alter table `file` alter column `parent_kind` drop default;

alter table `folder` rename column `parent_folder_id` to `parent_id`;
alter table `folder` add column `parent_kind` tinyint unsigned not null default 0;
alter table `folder` alter column `parent_kind` drop default;
alter table `folder` add column `depth` tinyint unsigned not null default 0;
alter table `folder` alter column `depth` drop default;

alter table `file`
    add key `idx_parent_name` (`parent_kind`, `parent_id`, `name`, `file_id`);

alter table `folder`
    add key `idx_parent_name` (`parent_kind`, `parent_id`, `name`, `folder_id`);

alter table `file_embedding`
    add key `idx_file_id` (`file_id`);

alter table `podcast`
    add key `idx_project_created_at` (`project_id`, `podcast_created_at`, `podcast_id`);
