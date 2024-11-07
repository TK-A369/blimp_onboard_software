pub trait BlimpAlgorithm<EventType> {
    fn send_event(ev: &EventType) -> impl std::future::Future<Output = ()>;
}
