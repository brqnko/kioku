create table `user` (
    `user_id` varbinary(16) not null,

    `display_name` varchar(32) not null,
    `language_code` varchar(7) not null,
    `joined_at` datetime not null,

    `iss` varchar(256) not null,
    `sub` varchar(256) not null,

    `recent_seen_file_ids` json not null,

    `ai_learning_summary` varchar(512) not null,
    `ai_learning_summary_updated_at` datetime not null,

    `created_at` datetime not null default(now()),
    `updated_at` datetime not null default(now()) on update now(),

    unique key (`iss`, `sub`),

    primary key (`user_id`) /*T![clustered_index] CLUSTERED */
);

create table `refresh_token` (
    `refresh_token_id` varbinary(16) not null,

    `user_id` varbinary(16) not null,
    `token_hash` varchar(256) not null,
    `generation` int not null,
    `ip_address` varchar(64) not null,
    `user_agent` varchar(512) not null,
    `access_token_jti` varbinary(16) not null,

    `activated_at` datetime not null,
    `last_used_at` datetime not null,
    `expires_at` datetime not null,

    `created_at` datetime not null default(now()),
    `updated_at` datetime not null default(now()) on update now(),

    primary key (`refresh_token_id`) /*T![clustered_index] CLUSTERED */
);

create table `file` (
    `file_id` varbinary(16) not null,

    `name` varchar(256) not null,
    `description` varchar(1024) not null,
    `user_id` varbinary(16) not null,

    `storage_id` varbinary(16) not null,
    `file_size` bigint unsigned not null,
    `parent_folder_id` varbinary(16) not null,

    `uploaded_at` datetime not null,
    `changed_at` datetime not null,

    `created_at` datetime not null default(now()),
    `updated_at` datetime not null default(now()) on update now(),

    primary key (`file_id`) /*T![clustered_index] CLUSTERED */
);

create table `file_embedding` (
    `file_embedding_id` varbinary(16) not null,

    `file_id` varbinary(16) not null,

    `original_text` varchar(512) not null,
    `embedding` vector(1024) not null,

    `indexed_at` datetime not null,

    `created_at` datetime not null default(now()),
    `updated_at` datetime not null default(now()) on update now(),

    primary key (`file_embedding_id`) /*T![clustered_index] CLUSTERED */
);

create table `folder` (
    `folder_id` varbinary(16) not null,

    `parent_folder_id` varbinary(16) not null,

    `name` varchar(256) not null,
    `description` varchar(1024) not null,
    `user_id` varbinary(16) not null,

    `uploaded_at` datetime not null,
    `changed_at` datetime not null,

    `created_at` datetime not null default(now()),
    `updated_at` datetime not null default(now()) on update now(),

    primary key (`folder_id`) /*T![clustered_index] CLUSTERED */
);

create table `podcast` (
    `podcast_id` varbinary(16) not null,

    `name` varchar(256) not null,
    `description` varchar(1024) not null,
    `user_id` varbinary(16) not null,

    `used_file_ids` json not null,
    `podcast_script` json not null,

    `podcast_created_at` datetime not null,

    `created_at` datetime not null default(now()),
    `updated_at` datetime not null default(now()) on update now(),

    primary key (`podcast_id`) /*T![clustered_index] CLUSTERED */
);

create table `chat` (
    `chat_id` varbinary(16) not null,

    `name` varchar(256) not null,
    `user_id` varbinary(16) not null,
    `messages` json not null,

    `started_at` datetime not null,
    `last_activity_at` datetime not null,

    `created_at` datetime not null default(now()),
    `updated_at` datetime not null default(now()) on update now(),

    primary key (`chat_id`) /*T![clustered_index] CLUSTERED */
);

create table `chat_file` (
    `chat_file_id` varbinary(16) not null,

    `name` varchar(256) not null,
    `chat_id` varbinary(16) not null,
    `user_id` varbinary(16) not null,

    `storage_id` varbinary(16) not null,
    `file_size` bigint unsigned not null,

    `uploaded_at` datetime not null,
    `changed_at` datetime not null,

    `created_at` datetime not null default(now()),
    `updated_at` datetime not null default(now()) on update now(),

    primary key (`chat_file_id`) /*T![clustered_index] CLUSTERED */
);
