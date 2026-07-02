//! 配置文件热重载：监听配置文件变化并通知订阅者。
//!
//! 使用 `notify` crate 监听文件系统事件，经过去抖后通过 [`ConfigNotifier`] 广播。

use crate::notifier::ConfigNotifier;
use crate::storage::ConfigStorage;
use notify::{Config as NotifyConfig, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::MissedTickBehavior;

/// 文件变更去抖动间隔。
const DEBOUNCE_INTERVAL: Duration = Duration::from_millis(300);

/// 配置热重载监听器。
///
/// 负责把配置文件改动转换为 [`ConfigNotifier`] 的更新。
pub struct ConfigWatcher {
    storage: ConfigStorage,
    notifier: ConfigNotifier,
}

impl ConfigWatcher {
    /// 创建监听器。
    ///
    /// `notifier` 用于把重新加载后的配置广播给订阅者。
    pub fn new(storage: ConfigStorage, notifier: ConfigNotifier) -> Self {
        Self { storage, notifier }
    }

    /// 启动异步监听循环。
    ///
    /// 返回的 `JoinHandle` 可用于等待任务结束；调用方通常不等待，而是与主循环并发运行。
    /// 当 `stop_signal` 被 drop 或发送消息时，监听循环会优雅退出。
    pub fn spawn(
        self,
        stop_signal: tokio::sync::oneshot::Receiver<()>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            if let Err(e) = self.run(stop_signal).await {
                tracing::error!("config watcher error: {}", e);
            }
        })
    }

    /// 监听循环核心逻辑。
    async fn run(self, mut stop_signal: tokio::sync::oneshot::Receiver<()>) -> crate::Result<()> {
        let (tx, mut rx) = mpsc::channel::<notify::Result<Event>>(16);
        let config_path = self.storage.path().to_path_buf();
        let parent_dir = config_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| config_path.clone());

        // 使用 `spawn_blocking` 包装同步文件系统 watcher，避免阻塞异步 runtime。
        let _parent_dir_for_spawn = parent_dir.clone();
        let mut watcher = tokio::task::spawn_blocking(move || {
            RecommendedWatcher::new(
                move |res| {
                    let _ = tx.blocking_send(res);
                },
                NotifyConfig::default(),
            )
        })
        .await
        .map_err(|e| crate::ConfigError::Validation(format!("watcher spawn: {}", e)))??;

        watcher.watch(&parent_dir, RecursiveMode::NonRecursive)?;

        // 去抖定时器：文件事件密集时只触发一次重载。
        let mut debounce = tokio::time::interval(DEBOUNCE_INTERVAL);
        debounce.set_missed_tick_behavior(MissedTickBehavior::Delay);
        debounce.reset();

        // 记录是否有一个待处理的重载请求。
        let mut pending_reload = false;
        // 最后一次重载时间，用于跳过事件风暴。
        let mut last_reload = tokio::time::Instant::now();

        loop {
            tokio::select! {
                _ = &mut stop_signal => {
                    tracing::info!("config watcher received stop signal, exiting");
                    break;
                }
                Some(res) = rx.recv() => {
                    match res {
                        Ok(event) => {
                            if is_config_event(&event, &config_path) {
                                pending_reload = true;
                                debounce.reset();
                            }
                        }
                        Err(e) => {
                            tracing::warn!("notify error: {}", e);
                        }
                    }
                }
                _ = debounce.tick() => {
                    if pending_reload {
                        let now = tokio::time::Instant::now();
                        if now.duration_since(last_reload) >= DEBOUNCE_INTERVAL {
                            self.reload_and_notify().await;
                            last_reload = now;
                        }
                        pending_reload = false;
                    }
                }
            }
        }

        Ok(())
    }

    /// 重新加载配置并广播。
    async fn reload_and_notify(&self) {
        match self.storage.load().await {
            Ok(config) => {
                if let Err(e) = self.notifier.update(config) {
                    tracing::warn!("failed to broadcast reloaded config: {}", e);
                } else {
                    tracing::info!("config reloaded and broadcast");
                }
            }
            Err(e) => {
                tracing::warn!("failed to reload config: {}", e);
            }
        }
    }
}

/// 判断一个 notify 事件是否指向目标配置文件。
fn is_config_event(event: &Event, config_path: &Path) -> bool {
    event.paths.iter().any(|p| p == config_path)
        && matches!(
            event.kind,
            notify::EventKind::Modify(_)
                | notify::EventKind::Create(_)
                | notify::EventKind::Remove(_)
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::AppConfig;
    use crate::storage::ConfigStorage;
    use std::time::Instant;
    use tokio::time::timeout;

    #[tokio::test(flavor = "multi_thread")]
    async fn watcher_detects_external_change() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let storage = ConfigStorage::new(&path);
        storage.save(&AppConfig::default_config()).await.unwrap();

        let notifier = ConfigNotifier::new(AppConfig::default_config());
        let mut rx = notifier.subscribe();
        // 跳过初始快照。
        let _ = rx.borrow_and_update();

        let watcher = ConfigWatcher::new(storage, notifier);
        let (tx, rx_stop) = tokio::sync::oneshot::channel::<()>();
        let handle = watcher.spawn(rx_stop);

        // 等待 watcher 初始化完成。
        tokio::time::sleep(Duration::from_millis(200)).await;

        // 外部修改配置文件。
        let mut updated = AppConfig::default_config();
        updated.active_profile = "game_a".to_string();
        updated
            .profiles
            .insert("game_a".to_string(), updated.profiles["default"].clone());
        let storage2 = ConfigStorage::new(&path);
        storage2.save(&updated).await.unwrap();

        // 等待广播，最多 5 秒。
        let start = Instant::now();
        let received = timeout(Duration::from_secs(5), async {
            loop {
                rx.changed().await.ok()?;
                let snap = rx.borrow().clone();
                if snap.active_profile == "game_a" {
                    return Some(());
                }
            }
        })
        .await
        .ok()
        .flatten();

        let _ = tx.send(());
        let _ = handle.await;

        assert!(
            received.is_some(),
            "did not receive updated config within 5s (elapsed {:?})",
            start.elapsed()
        );
    }
}
