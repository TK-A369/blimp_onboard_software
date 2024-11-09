pub trait BlimpAlgorithm<EventType, ActionType> {
    pub fn handle_event(&mut self, ev: &EventType) -> impl std::future::Future<Output = ()>;
    pub fn set_action_callback(&mut self, callback: Box<dyn Fn(ActionType) -> ()>);
}
