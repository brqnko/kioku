#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct PodcastRequest {
    pub podcast_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub project_id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub used_file_ids: Vec<uuid::Uuid>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub voice_style: String,
    #[serde(default)]
    pub voice_style_2: Option<String>,
    pub length: String,
}

#[async_trait::async_trait]
pub trait PodcastRequestService: Send + Sync {
    async fn save(
        &self,
        request: &PodcastRequest,
        ttl: std::time::Duration,
    ) -> Result<(), anyhow::Error>;
    async fn get(&self, podcast_id: uuid::Uuid) -> Result<Option<PodcastRequest>, anyhow::Error>;
    async fn list_by_user(&self, user_id: uuid::Uuid)
    -> Result<Vec<PodcastRequest>, anyhow::Error>;
    async fn remove(&self, podcast_id: uuid::Uuid) -> Result<(), anyhow::Error>;
}

pub struct PodcastRequestServiceImpl {
    pool: deadpool_redis::Pool,
}

impl PodcastRequestServiceImpl {
    pub fn new(pool: deadpool_redis::Pool) -> Self {
        Self { pool }
    }

    fn request_key(podcast_id: uuid::Uuid) -> String {
        format!("kioku:podcast_request:{podcast_id}")
    }

    fn user_index_key(user_id: uuid::Uuid) -> String {
        format!("kioku:user:{user_id}:podcast_requests")
    }
}

#[async_trait::async_trait]
impl PodcastRequestService for PodcastRequestServiceImpl {
    async fn save(
        &self,
        request: &PodcastRequest,
        ttl: std::time::Duration,
    ) -> Result<(), anyhow::Error> {
        use redis::AsyncCommands as _;

        let payload = serde_json::to_string(request)?;
        let ttl_secs = ttl.as_secs();

        let mut conn = self.pool.get().await?;
        conn.set_ex::<_, _, ()>(Self::request_key(request.podcast_id), payload, ttl_secs)
            .await?;
        conn.sadd::<_, _, ()>(
            Self::user_index_key(request.user_id),
            request.podcast_id.to_string(),
        )
        .await?;
        conn.expire::<_, ()>(Self::user_index_key(request.user_id), ttl_secs as i64)
            .await?;
        Ok(())
    }

    async fn get(&self, podcast_id: uuid::Uuid) -> Result<Option<PodcastRequest>, anyhow::Error> {
        use redis::AsyncCommands as _;

        let mut conn = self.pool.get().await?;
        let raw = conn
            .get::<_, Option<String>>(Self::request_key(podcast_id))
            .await?;
        match raw {
            Some(s) => Ok(Some(serde_json::from_str::<PodcastRequest>(&s)?)),
            None => Ok(None),
        }
    }

    async fn list_by_user(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Vec<PodcastRequest>, anyhow::Error> {
        let script = redis::Script::new(
            r#"
            local ids = redis.call('SMEMBERS', KEYS[1])
            local result = {}
            local stale = {}
            for _, id in ipairs(ids) do
                local val = redis.call('GET', ARGV[1] .. id)
                if val then
                    table.insert(result, val)
                else
                    table.insert(stale, id)
                end
            end
            if #stale > 0 then
                redis.call('SREM', KEYS[1], unpack(stale))
            end
            return result
            "#,
        );

        let mut conn = self.pool.get().await?;
        let payloads = script
            .key(Self::user_index_key(user_id))
            .arg("kioku:podcast_request:")
            .invoke_async::<Vec<String>>(&mut *conn)
            .await?;

        payloads
            .into_iter()
            .map(|s| serde_json::from_str::<PodcastRequest>(&s).map_err(anyhow::Error::from))
            .collect::<Result<Vec<PodcastRequest>, anyhow::Error>>()
    }

    async fn remove(&self, podcast_id: uuid::Uuid) -> Result<(), anyhow::Error> {
        use redis::AsyncCommands as _;

        let mut conn = self.pool.get().await?;
        let raw = conn
            .get::<_, Option<String>>(Self::request_key(podcast_id))
            .await?;
        conn.del::<_, ()>(Self::request_key(podcast_id)).await?;
        if let Some(s) = raw {
            let req = serde_json::from_str::<PodcastRequest>(&s)?;
            conn.srem::<_, _, ()>(Self::user_index_key(req.user_id), podcast_id.to_string())
                .await?;
        }
        Ok(())
    }
}
