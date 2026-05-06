alter table `refresh_token`
    drop key `uk_token_hash`,
    drop key `idx_user_id`;

alter table `project`
    drop key `idx_created_by_last_seen_at_desc`,
    drop key `idx_created_by_last_seen_at_asc`;
