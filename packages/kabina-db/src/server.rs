#[salsa::input]
#[derive(Debug, Clone)]
pub struct Server {
	name: String,
}
