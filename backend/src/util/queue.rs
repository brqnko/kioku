#[derive(serde::Serialize, serde::Deserialize)]
pub enum Job {
    IndexFile { file_id: uuid::Uuid },
}

impl Job {
    pub fn task_id(&self) -> String {
        let key = match self {
            Self::IndexFile { file_id } => format!("IndexFile:{file_id}"),
        };
        sha256::digest(key)
    }
}

pub struct DequeuedJob {
    pub receipt: String,
    pub task_id: String,
    pub job: Job,
}

#[async_trait::async_trait]
pub trait QueueService: Send + Sync {
    async fn enqueue(&self, job: Job) -> Result<(), anyhow::Error>;
    async fn dequeue(&self) -> Result<Option<DequeuedJob>, anyhow::Error>;
    async fn ack(&self, receipt: &str, task_id: &str) -> Result<(), anyhow::Error>;
}

pub struct QueueServiceImpl {
    pool: deadpool_redis::Pool,
    consumer_name: String,
}

impl QueueServiceImpl {
    const STREAM_KEY: &'static str = "kioku:jobs";
    const GROUP_NAME: &'static str = "workers";
    const DEDUP_TTL_SECS: u64 = 3600;

    pub async fn new(pool: deadpool_redis::Pool) -> Result<Self, anyhow::Error> {
        let consumer_name = uuid::Uuid::new_v4().to_string();

        let mut conn = pool.get().await?;
        let result: Result<(), redis::RedisError> = redis::cmd("XGROUP")
            .arg("CREATE")
            .arg(Self::STREAM_KEY)
            .arg(Self::GROUP_NAME)
            .arg("$")
            .arg("MKSTREAM")
            .query_async(&mut conn)
            .await;

        if let Err(err) = result {
            let msg = err.to_string();
            if !msg.contains("BUSYGROUP") {
                return Err(anyhow::anyhow!("XGROUP CREATE failed: {msg}"));
            }
        }

        Ok(Self {
            pool,
            consumer_name,
        })
    }

    fn dedup_key(task_id: &str) -> String {
        format!("kioku:dedup:{task_id}")
    }
}

#[async_trait::async_trait]
impl QueueService for QueueServiceImpl {
    async fn enqueue(&self, job: Job) -> Result<(), anyhow::Error> {
        let task_id = job.task_id();
        let payload = serde_json::to_string(&job)?;

        let mut conn = self.pool.get().await?;

        let set: Option<String> = redis::cmd("SET")
            .arg(Self::dedup_key(&task_id))
            .arg("1")
            .arg("NX")
            .arg("EX")
            .arg(Self::DEDUP_TTL_SECS)
            .query_async(&mut conn)
            .await?;

        if set.is_none() {
            tracing::debug!(task_id = %task_id, "enqueue skipped (already in flight)");
            return Ok(());
        }

        let _: String = redis::cmd("XADD")
            .arg(Self::STREAM_KEY)
            .arg("*")
            .arg("task_id")
            .arg(&task_id)
            .arg("payload")
            .arg(payload)
            .query_async(&mut conn)
            .await?;

        Ok(())
    }

    async fn dequeue(&self) -> Result<Option<DequeuedJob>, anyhow::Error> {
        let mut conn = self.pool.get().await?;
        let response: redis::streams::StreamReadReply = match redis::cmd("XREADGROUP")
            .arg("GROUP")
            .arg(Self::GROUP_NAME)
            .arg(&self.consumer_name)
            .arg("COUNT")
            .arg(1)
            .arg("BLOCK")
            .arg(30000)
            .arg("STREAMS")
            .arg(Self::STREAM_KEY)
            .arg(">")
            .query_async(&mut conn)
            .await
        {
            Ok(r) => r,
            Err(err) if err.is_timeout() => return Ok(None),
            Err(err) => return Err(err.into()),
        };

        for stream in response.keys {
            for entry in stream.ids {
                let task_id_value = match entry.map.get("task_id") {
                    Some(v) => v,
                    None => {
                        return Err(anyhow::anyhow!(
                            "missing task_id field in stream entry: {}",
                            entry.id
                        ));
                    }
                };
                let task_id: String =
                    redis::FromRedisValue::from_redis_value(task_id_value.clone())?;

                let payload_value = match entry.map.get("payload") {
                    Some(v) => v,
                    None => {
                        return Err(anyhow::anyhow!(
                            "missing payload field in stream entry: {}",
                            entry.id
                        ));
                    }
                };
                let payload: String =
                    redis::FromRedisValue::from_redis_value(payload_value.clone())?;
                let job: Job = serde_json::from_str(&payload)?;

                return Ok(Some(DequeuedJob {
                    receipt: entry.id,
                    task_id,
                    job,
                }));
            }
        }

        Ok(None)
    }

    async fn ack(&self, receipt: &str, task_id: &str) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.get().await?;
        let _: i64 = redis::cmd("XACK")
            .arg(Self::STREAM_KEY)
            .arg(Self::GROUP_NAME)
            .arg(receipt)
            .query_async(&mut conn)
            .await?;
        let _: i64 = redis::cmd("XDEL")
            .arg(Self::STREAM_KEY)
            .arg(receipt)
            .query_async(&mut conn)
            .await?;
        let _: i64 = redis::cmd("DEL")
            .arg(Self::dedup_key(task_id))
            .query_async(&mut conn)
            .await?;
        Ok(())
    }
}

pub async fn run(app: std::sync::Arc<crate::app::App>) {
    loop {
        let dequeued = match app.queue_service.dequeue().await {
            Ok(Some(ok)) => ok,
            Ok(None) => continue,
            Err(err) => {
                tracing::error!(?err, "dequeue failed");
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                continue;
            }
        };

        if let Err(err) = dispatch(&app, dequeued.job).await {
            tracing::error!(?err, receipt = %dequeued.receipt, "job processing failed");
            continue;
        }

        if let Err(err) = app
            .queue_service
            .ack(&dequeued.receipt, &dequeued.task_id)
            .await
        {
            tracing::error!(?err, receipt = %dequeued.receipt, "ack failed");
        }
    }
}

pub async fn dispatch(app: &crate::app::App, job: Job) -> Result<(), anyhow::Error> {
    match job {
        Job::IndexFile { file_id } => {
            let input = crate::features::file::usecase::IndexFileInput { file_id };
            match crate::features::file::usecase::index_file(app, input).await? {
                Ok(_) => Ok(()),
                Err(err) => Err(anyhow::anyhow!("index_file failed: {:?}", err)),
            }
        }
    }
}
