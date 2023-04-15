use crate::Binary;

#[salsa::input]
#[derive(Debug, Clone)]
pub struct Service {
	pub name: String,
	pub binary: Binary,
}
