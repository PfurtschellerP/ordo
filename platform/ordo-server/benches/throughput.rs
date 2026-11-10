//! Throughput of the server path: hub (verify, order, chain) + SQLite (one transaction
//! per message). Run with: `cargo bench -p ordo-server` (set TMPDIR to measure a real disk).

use std::time::Instant;

use ordo_core::{Author, DeviceId, IncidentId, KeyId, ServerName, Ulid, UserId};
use ordo_crypto::EventSigner;
use ordo_crypto::ed25519_dalek::SigningKey;
use ordo_crypto::seal::DataKey;
use ordo_server::store::Store;
use ordo_sync::message::ClientMessage;
use ordo_sync::{Hub, HubConfig, Replica, StaticPolicy};

fn entropy(seed: u64) -> impl FnMut(&mut [u8]) + Send {
    let mut state = seed | 1;
    move |b: &mut [u8]| {
        for x in b {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            *x = (state >> 24) as u8;
        }
    }
}

async fn run(total: usize, batch: usize) -> f64 {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("ordo.db")).await.unwrap();
    let master = DataKey::from_bytes([1; 32]);
    let server: ServerName = "ordo.example.org".parse().unwrap();
    let user = UserId::new("ben", &server).unwrap();
    let t0 = 1_762_600_000_000;
    let incident = IncidentId::new(Ulid::from_parts(t0, 1).unwrap(), &server);
    let device = DeviceId(Ulid::from_parts(t0, 2).unwrap());
    let key = SigningKey::from_bytes(&[3; 32]);
    let mut hub = Hub::new(
        HubConfig {
            server: server.clone(),
            protected: vec![],
            max_clock_ahead_ms: u64::MAX,
            vehicles: Vec::new(),
        },
        StaticPolicy::default().allow(&user, &["incident"]),
        EventSigner::new(
            SigningKey::from_bytes(&[9; 32]),
            KeyId::new("s", &server).unwrap(),
        ),
        Author {
            user: UserId::new("sys", &server).unwrap(),
            device: DeviceId(Ulid::from_parts(t0, 9).unwrap()),
        },
        entropy(9),
    );
    hub.register_device(device, user.clone(), key.verifying_key());
    let mut replica = Replica::new(
        user,
        device,
        EventSigner::new(key, KeyId::new("d", &server).unwrap()),
        entropy(3),
    );
    replica.subscribe(&incident, "incident");
    for out in hub.connect(1) {
        replica.receive(out.message, t0).unwrap();
    }
    let hello = replica.hello().expect("answer to the challenge");
    for out in hub.handle(1, hello, t0) {
        replica.receive(out.message, t0).unwrap();
    }
    // Create the events up front (device-side work is not measured here).
    let mut now = t0;
    for i in 0..total {
        now += 1;
        let payload = format!("{{\"unit\":\"RTW {i}\",\"status\":\"3\"}}");
        replica
            .record(
                &incident,
                "incident",
                "units",
                "units.statusChanged",
                1,
                payload.as_bytes(),
                now,
            )
            .unwrap();
    }
    let Some(ClientMessage::Push(events)) = replica.push() else {
        unreachable!()
    };
    let batches: Vec<Vec<_>> = events.chunks(batch).map(<[_]>::to_vec).collect();
    let start = Instant::now();
    for events in batches {
        let _ = hub.handle(1, ClientMessage::Push(events), now);
        let changes = hub.take_changes();
        store.save(&changes, &master).await.unwrap();
    }
    let secs = start.elapsed().as_secs_f64();
    assert_eq!(
        hub.stream(&incident, &"incident".parse().unwrap()).len(),
        total
    );
    total as f64 / secs
}

fn main() {
    let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
    for batch in [1, 10, 100] {
        let rate = runtime.block_on(run(5_000, batch));
        println!("batch {batch:>3}: {rate:>8.0} events/s");
    }
}
