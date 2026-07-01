//! 配置变更广播机制。
//!
//! 使用 `tokio::sync::watch` 实现多订阅者广播：配置更新后，所有订阅者收到新的
//! [`ConfigSnapshot`]。

use crate::schema::AppConfig;
use std::sync::Arc;
use tokio::sync::watch;

/// 配置快照，通过 `Arc` 共享，避免每次更新都深拷贝整个配置。
pub type ConfigSnapshot = Arc<AppConfig>;

/// 配置变更广播器。
///
/// 典型用法：
/// - UI 或 Watcher 调用 [`ConfigNotifier::update`] 推送新配置；
/// - Renderer 调用 [`ConfigNotifier::subscribe`] 接收变更。
#[derive(Debug, Clone)]
pub struct ConfigNotifier {
    sender: watch::Sender<ConfigSnapshot>,
}

impl ConfigNotifier {
    /// 从当前配置创建广播器。
    pub fn new(config: AppConfig) -> Self {
        let (sender, _receiver) = watch::channel(Arc::new(config));
        Self { sender }
    }

    /// 推送新的配置快照。
    ///
    /// 若所有接收端均已关闭，则返回错误。
    pub fn update(&self, config: AppConfig) -> crate::Result<()> {
        let snapshot = Arc::new(config);
        self.sender
            .send(snapshot)
            .map_err(|_| crate::ConfigError::NotifierClosed)
    }

    /// 订阅配置变更。
    ///
    /// 返回的 receiver 会立即收到当前最新快照，之后每次 `update` 都会再次触发。
    pub fn subscribe(&self) -> watch::Receiver<ConfigSnapshot> {
        self.sender.subscribe()
    }

    /// 当前活跃订阅者数量。
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::AppConfig;

    #[tokio::test]
    async fn subscriber_receives_updates() {
        let notifier = ConfigNotifier::new(AppConfig::default_config());
        let mut rx = notifier.subscribe();

        // 首次订阅立即收到当前快照。
        {
            let snapshot = rx.borrow_and_update();
            assert_eq!(snapshot.active_profile, "default");
        }

        let mut updated = AppConfig::default_config();
        updated.active_profile = "game_a".to_string();
        updated
            .profiles
            .insert("game_a".to_string(), updated.profiles["default"].clone());

        notifier.update(updated.clone()).unwrap();
        rx.changed().await.unwrap();
        {
            let snapshot = rx.borrow();
            assert_eq!(snapshot.active_profile, "game_a");
        }
    }

    #[test]
    fn subscriber_count_tracks_receivers() {
        let notifier = ConfigNotifier::new(AppConfig::default_config());
        assert_eq!(notifier.subscriber_count(), 0);

        let _rx1 = notifier.subscribe();
        assert_eq!(notifier.subscriber_count(), 1);

        let _rx2 = notifier.subscribe();
        assert_eq!(notifier.subscriber_count(), 2);
    }
}
