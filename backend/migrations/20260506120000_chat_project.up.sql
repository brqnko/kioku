alter table `chat`
    add column `project_id` varbinary(16) not null default 0x00000000000000000000000000000000;

alter table `chat`
    alter column `project_id` drop default;

alter table `chat`
    add key `idx_project_last_activity` (`project_id`, `last_activity_at`, `chat_id`);
