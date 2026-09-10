use libp2p::{
    gossipsub,
    mdns,
    noise,
    swarm::NetworkBehaviour,
    tcp,
    yamux,
    PeerId,
    Transport,
};
use libp2p::swarm::SwarmBuilder;
use libp2p::swarm::Swarm;
use libp2p::core::upgrade;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "GhostEvent")]
pub struct GhostBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
}

pub enum GhostEvent {
    Gossipsub(gossipsub::Event),
    Mdns(mdns::Event),
}

impl From<gossipsub::Event> for GhostEvent {
    fn from(e: gossipsub::Event) -> Self {
        Self::Gossipsub(e)
    }
}

impl From<mdns::Event> for GhostEvent {
    fn from(e: mdns::Event) -> Self {
        Self::Mdns(e)
    }
}

pub async fn build_swarm() -> Result<Swarm<GhostBehaviour>, Box<dyn std::error::Error>> {
    let id_keys = libp2p::identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(id_keys.public());

    let transport = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
        .upgrade(upgrade::Version::V1)
        .authenticate(noise::Config::new(&id_keys).expect("signing libp2p-noise static keypair"))
        .multiplex(yamux::Config::default())
        .boxed();

    let message_id_fn = |message: &gossipsub::Message| {
        let mut s = DefaultHasher::new();
        message.data.hash(&mut s);
        gossipsub::MessageId::from(s.finish().to_string())
    };

    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(10))
        .validation_mode(gossipsub::ValidationMode::Strict)
        .message_id_fn(message_id_fn)
        .build().expect("Valid config");

    let mut gossipsub = gossipsub::Behaviour::new(gossipsub::MessageAuthenticity::Signed(id_keys.clone()), gossipsub_config).expect("Correct config");
    gossipsub.subscribe(&gossipsub::IdentTopic::new("ghostchat-global"))?;

    let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)?;
    let behaviour = GhostBehaviour { gossipsub, mdns };
    
    let mut swarm = SwarmBuilder::with_tokio_executor(
        transport,
        behaviour,
        peer_id,
    ).build();
    
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
    Ok(swarm)
}
