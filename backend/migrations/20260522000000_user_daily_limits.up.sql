alter table `user`
    add column `podcast_daily_count` int unsigned not null default 0,
    add column `podcast_daily_count_reset_at` datetime not null default(now()),
    add column `chatbot_daily_count` int unsigned not null default 0,
    add column `chatbot_daily_count_reset_at` datetime not null default(now()),
    add column `file_upload_daily_count` int unsigned not null default 0,
    add column `file_upload_daily_count_reset_at` datetime not null default(now());

alter table `user`
    alter column `podcast_daily_count` drop default,
    alter column `podcast_daily_count_reset_at` drop default,
    alter column `chatbot_daily_count` drop default,
    alter column `chatbot_daily_count_reset_at` drop default,
    alter column `file_upload_daily_count` drop default,
    alter column `file_upload_daily_count_reset_at` drop default;
