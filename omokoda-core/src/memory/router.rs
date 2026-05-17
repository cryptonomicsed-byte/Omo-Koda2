pub struct MemoryRouter;

impl MemoryRouter {
    pub fn route_content(_content: &str, is_private: bool) -> MemoryRoute {
        if is_private {
            MemoryRoute::Private
        } else {
            MemoryRoute::Public
        }
    }
}

pub enum MemoryRoute {
    Public,
    Private,
}
