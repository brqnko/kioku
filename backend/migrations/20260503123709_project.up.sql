create table `project` (
    `project_id` varbinary(16) not null,

    `created_by` varbinary(16) not null,

    `name` varchar(256) not null,
    `description` varchar(512) not null,

    `indexed_at` datetime not null default(now()),
    `last_seen_at` datetime not null default(now()),

    `last_seen_file_id` varbinary(16) not null,

    `created_at` datetime not null default(now()),
    `updated_at` datetime not null default(now()) on update now(),

    primary key (`project_id`) /*T![clustered_index] CLUSTERED */
)