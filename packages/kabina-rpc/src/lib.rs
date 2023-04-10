use url::Url;

#[tarpc::service]
pub trait Kabina {
    async fn hello(name: String) -> String;
    async fn version() -> String;
    async fn schema_run(url: Url);
    async fn terminate();
}
