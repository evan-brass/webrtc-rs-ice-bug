use eyre::Result;
use std::sync::Arc;
use webrtc_ice::agent::agent_config::AgentConfig;
use webrtc_ice::candidate::CandidateType;
use webrtc_ice::mdns::MulticastDnsMode;
use webrtc_ice::network_type::NetworkType;
use webrtc_ice::agent::Agent;

#[tokio::main]
async fn main() -> Result<()> {
	simple_logger::init_with_env()?;

	/*
	 * Create the first agent which will be an ice-lite server.
	 */
	let mut config_1 = AgentConfig::default();
	config_1.multicast_dns_mode = MulticastDnsMode::Disabled;
	config_1.candidate_types = vec![CandidateType::Host];
	config_1.network_types = vec![NetworkType::Udp4];
	config_1.is_controlling = false;
	config_1.lite = true;
	config_1.local_ufrag = "Peer1".to_owned();

	let agent_1 = Arc::new(Agent::new(config_1).await?);

	/*
	 * Create the second agent which will be an ice full client
	 */
	let mut config_2 = AgentConfig::default();
	config_2.multicast_dns_mode = MulticastDnsMode::Disabled;
	config_2.candidate_types = vec![CandidateType::Host];
	config_2.network_types = vec![NetworkType::Udp4];
	config_2.is_controlling = true;
	config_2.local_ufrag = "Peer2".to_owned();

	let agent_2 = Arc::new(Agent::new(config_2).await?);

	// Handle ICE candidates
	let agent_2_callback = agent_2.clone();
	agent_1.on_candidate(Box::new(move |candidate| {
		let agent_2 = agent_2_callback.clone();
		Box::pin(async move {
			if let Some(candidate) = candidate {
				println!("Agent 1 - {}:{}", candidate.address(), candidate.port());
				agent_2.add_remote_candidate(&candidate).await.unwrap();
			}
		})
	})).await;
	agent_2.on_candidate(Box::new(|candidate| {
		if let Some(candidate) = candidate {
			println!("Agent 2 - {}:{}", candidate.address(), candidate.port());
		}
		Box::pin(async {})
	})).await;

	agent_1.gather_candidates().await?;
	agent_2.gather_candidates().await?;

	let (agent_2_ufrag, agent_2_pwd) = agent_2.get_local_user_credentials().await;
	let (agent_1_ufrag, agent_1_pwd) = agent_1.get_local_user_credentials().await;
	println!("{agent_1_ufrag}:{agent_2_ufrag} {agent_1_pwd}:{agent_2_pwd}");
	tokio::spawn(async move {
		let (_cancel_tx, cancel_rx) = tokio::sync::mpsc::channel(1);
		let _conn = agent_1.accept(cancel_rx, agent_2_ufrag, agent_2_pwd).await.unwrap();
		println!("Yay done 1!");
	});
	let (_cancel_tx, cancel_rx) = tokio::sync::mpsc::channel(1);
	let _conn = agent_2.dial(cancel_rx, agent_1_ufrag, agent_1_pwd).await?;
	println!("Yay done 2!");

	Ok(())
}
