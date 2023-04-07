#[tarpc::service]
pub trait World {
    async fn hello(name: String) -> String;
    async fn version() -> String;
    async fn terminate();
}
