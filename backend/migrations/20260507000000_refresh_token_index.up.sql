alter table `refresh_token`
    drop key `idx_user_id`,
    add key `idx_user_id_token_id` (`user_id`, `refresh_token_id`);
