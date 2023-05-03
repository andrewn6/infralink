use reqwest::Client;

pub struct SharedConfig {
	pub clients: ProviderClients,
}

pub struct ProviderClients {
	pub vultr: Option<Client>,
	pub hetzner: Option<Client>,
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

	pub fn hetzner(mut self) -> Client {
		if self.hetzner.is_none() {
			self.hetzner = Some(Client::builder().use_rustls_tls().build().unwrap());

			self.hetzner.unwrap()
		} else {
			self.hetzner.unwrap()
		}
	}
}
