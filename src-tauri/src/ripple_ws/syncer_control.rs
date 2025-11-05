pub trait SyncerControl {
    async fn start_syncer(&self) -> anyhow::Result<()>;
    async fn stop_syncer(&self) -> anyhow::Result<()>;
}
