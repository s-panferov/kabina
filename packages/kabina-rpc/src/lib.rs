#[tarpc::service]
pub trait Kabina {
    async fn hello(name: String) -> String;
    async fn version() -> String;
    async fn terminate();
}
