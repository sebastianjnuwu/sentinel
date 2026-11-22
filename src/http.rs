use std::sync::OnceLock;
use std::time::Duration;
use ureq::Agent;

static AGENT: OnceLock<Agent> = OnceLock::new();

pub fn agent() -> &'static Agent {
    AGENT.get_or_init(|| {
        ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(5))
            .timeout_read(Duration::from_secs(8))
            .max_idle_connections(32)
            .max_idle_connections_per_host(16)
            .build()
    })
}
