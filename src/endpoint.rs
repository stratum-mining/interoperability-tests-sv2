use std::{net::SocketAddr, time::Duration};

use integration_tests_sv2::utils::create_upstream;

use crate::{app_type::AppType, scenarios::ScenarioError};

const CONNECT_BUDGET: Duration = Duration::from_secs(3);

/// An Sv2 endpoint under test.
#[derive(Debug, Clone)]
pub struct Endpoint {
    pub addr: SocketAddr,
    pub app_type: AppType,
    pub user_identity: String,
}

impl Endpoint {
    /// Fails fast when the endpoint is unreachable or refuses the Noise
    /// handshake, instead of stalling every scenario on a message timeout.
    ///
    /// TCP connect retries within the budget: a freshly started local service
    /// may not be listening yet, while a dead endpoint refuses every attempt
    /// instantly. The handshake is then tried once on the open stream.
    pub async fn preflight(&self) -> Result<(), ScenarioError> {
        let deadline = std::time::Instant::now() + CONNECT_BUDGET;
        let stream = loop {
            match tokio::net::TcpStream::connect(self.addr).await {
                Ok(stream) => break stream,
                Err(e) => {
                    if std::time::Instant::now() >= deadline {
                        return Err(format!("endpoint unreachable: {e}").into());
                    }
                    tokio::time::sleep(Duration::from_millis(200)).await;
                }
            }
        };
        match create_upstream(stream).await {
            Some(_) => Ok(()),
            None => Err("Noise handshake with endpoint failed".into()),
        }
    }
}
