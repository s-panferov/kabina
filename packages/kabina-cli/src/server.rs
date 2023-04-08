use kabina_rpc::Kabina;
use tarpc::context::Context;

pub const VERSION: u32 = const_random::const_random!(u32);

// This is the type that implements the generated World trait. It is the business logic
// and is used to start the server.
#[derive(Clone)]
pub struct KabinaServer();

#[tarpc::server]
impl Kabina for KabinaServer {
    async fn hello(self, _: Context, name: String) -> String {
        format!("Hello, {name}! You are connected")
    }

    async fn version(self, _: Context) -> String {
        VERSION.to_string()
    }

    async fn terminate(self, _: Context) {
        let _ = std::fs::remove_file("/tmp/kabina.sock");
        std::process::exit(0)
    }
}
