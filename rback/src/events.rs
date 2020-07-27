#[derive(Debug)]
// GGPOEventCode
pub enum Event {
    ConnectedToPeer,
    SynchronizingWithPeer,
    SynchronizedWithPeer,
    Running,
    DisconnectedFromPeer,
    Timesync,
    ConnectionInterrupted,
    ConnectionResumed,
}
