alter table `podcast`
    add column `audio_storage_id` varbinary(16) not null default 0x00000000000000000000000000000000;

alter table `podcast`
    alter column `audio_storage_id` drop default;
