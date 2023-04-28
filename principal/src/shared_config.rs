use reqwest::Client;

pub struct SharedConfig {
	pub clients: ProviderClients,
}

pub struct ProviderClients {
	pub vultr: Option<Client>,
}

impl ProviderClients {
	pub fn vultr(mut self) -> Client {
		if self.vultr.is_none() {
			self.vultr = Some(Client::builder().use_rustls_tls().build().unwrap());

			self.vultr.unwrap()
		} else {
			self.vultr.unwrap()
		}
	}
}
