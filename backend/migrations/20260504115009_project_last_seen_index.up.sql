alter table `project`
    add key `idx_created_by_last_seen_at_desc` (`created_by`, `last_seen_at` desc, `project_id`),
    add key `idx_created_by_last_seen_at_asc` (`created_by`, `last_seen_at` asc, `project_id`);

alter table `refresh_token`
    add key `idx_user_id` (`user_id`),
    add unique key `uk_token_hash` (`token_hash`);
