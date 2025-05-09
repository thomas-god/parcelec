use std::time::Duration;

use tokio_util::sync::CancellationToken;

pub mod config;
pub mod units;

pub fn program_actors_termination(duration_before_cleaning: Duration) -> CancellationToken {
    let token = CancellationToken::new();
    let cloned_token = token.clone();

    tokio::spawn(async move {
        tokio::time::sleep(duration_before_cleaning).await;
        token.cancel();
    });

    cloned_token
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use crate::utils::program_actors_termination;

    #[tokio::test]
    async fn test_cleanup() {
        let token = program_actors_termination(Duration::from_millis(20));

        assert!(!token.is_cancelled());
        tokio::time::sleep(Duration::from_millis(25)).await;
        assert!(token.is_cancelled());
    }
}
