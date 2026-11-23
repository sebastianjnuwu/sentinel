use std::sync::OnceLock;
use std::time::Duration;
use ureq::Agent;

static AGENT: OnceLock<Agent> = OnceLock::new();

pub fn user_agent() -> &'static str {
    concat!("sentinel-deps-cli/", env!("CARGO_PKG_VERSION"))
}

pub fn agent() -> &'static Agent {
    AGENT.get_or_init(|| {
        let config = Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(15)))
            .timeout_connect(Some(Duration::from_secs(5)))
            .timeout_recv_response(Some(Duration::from_secs(10)))
            .max_idle_connections(32)
            .max_idle_connections_per_host(16)
            .build();
        config.into()
    })
}
