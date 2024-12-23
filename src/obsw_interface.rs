pub trait BlimpAlgorithm<EventType, ActionType> {
    fn handle_event(
        &mut self,
        ev: &EventType,
    ) -> std::pin::Pin<Box<impl std::future::Future<Output = ()>>>;
    fn set_action_callback(&mut self, callback: Box<dyn Fn(ActionType) -> () + Send>);
}
