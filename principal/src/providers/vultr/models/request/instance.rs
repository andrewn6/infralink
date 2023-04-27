use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::shared_config::SharedConfig;
use dotenv_codegen::dotenv;

use super::bandwidth::Bandwidth;
use super::plan::Plan;
use super::region::Region;

use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Instance {
	pub id: String,
	pub os: String,
	pub ram: u32,
	pub disk: u32,
	pub main_ip: String,
	pub vcpu_count: u32,
	pub region: Region,
	pub default_password: String,
	pub date_created: String,
	pub status: String,
	pub power_status: String,
	pub server_status: String,
	pub allowed_bandwidth: u32,
	pub netmask_v4: String,
	pub gateway_v4: String,
	pub v6_networks: Vec<HashMap<String, String>>,
	pub hostname: String,
	pub label: String,
	pub tag: Option<String>,
	pub internal_ip: Option<String>,
	pub kvm: String,
	pub os_id: u32,
	pub app_id: Option<u32>,
	pub image_id: Option<String>,
	pub firewall_group_id: Option<String>,
	pub features: Vec<String>,
	pub plan: Plan,
	pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InstanceBuilder {
	pub region: Region,
	pub plan: Plan,
	pub os_id: Option<u32>,
	pub ipxe_chain_url: Option<String>,
	pub iso_id: Option<String>,
	pub script_id: Option<String>,
	pub snapshot_id: Option<String>,
	pub enable_ipv6: Option<bool>,
	#[serde(rename = "attach_private_network")]
	pub attach_private_network_deprecated: Option<Vec<String>>,
	pub attach_vpc: Option<Vec<String>>,
	pub label: Option<String>,
	pub sshkey_id: Option<Vec<String>>,
	pub backups: Option<String>,
	pub app_id: Option<u32>,
	pub image_id: Option<String>,
	pub user_data: Option<String>,
	pub ddos_protection: Option<bool>,
	pub activation_email: Option<bool>,
	pub hostname: Option<String>,
	pub tag: Option<String>,
	pub firewall_group_id: Option<String>,
	pub reserved_ipv4: Option<String>,
	#[serde(rename = "enable_private_network")]
	pub enable_private_network_deprecated: Option<bool>,
	pub enable_vpc: Option<bool>,
	pub tags: Option<Vec<String>>,
}

pub enum InstanceType {
	HighPerformance,
	HighFrequency,
	GeneralPurpose,
	CPUOptimized,
}

impl ToString for InstanceType {
	fn to_string(&self) -> String {
		match self {
			InstanceType::HighPerformance => "vhp".to_string(),
			InstanceType::HighFrequency => "vhf".to_string(),
			InstanceType::GeneralPurpose => "voc-g".to_string(),
			InstanceType::CPUOptimized => "voc-c".to_string(),
		}
	}
}

impl FromStr for InstanceType {
	type Err = &'static str;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"vhp" => Ok(InstanceType::HighPerformance),
			"vhf" => Ok(InstanceType::HighFrequency),
			"voc-g" => Ok(InstanceType::GeneralPurpose),
			"voc-c" => Ok(InstanceType::CPUOptimized),
			_ => Err("Invalid instance type"),
		}
	}
}

impl InstanceBuilder {
	pub fn new() -> Self {
		InstanceBuilder {
			region: Region::Unknown,
			plan: Plan::Unknown,
			os_id: None,
			ipxe_chain_url: None,
			iso_id: None,
			script_id: None,
			snapshot_id: None,
			enable_ipv6: None,
			attach_private_network_deprecated: None,
			attach_vpc: None,
			label: None,
			sshkey_id: None,
			backups: None,
			app_id: None,
			image_id: None,
			user_data: None,
			ddos_protection: None,
			activation_email: None,
			hostname: None,
			tag: None,
			firewall_group_id: None,
			reserved_ipv4: None,
			enable_private_network_deprecated: None,
			enable_vpc: None,
			tags: None,
		}
	}

	pub fn region(mut self, region: Region) -> Self {
		self.region = region;
		self
	}

	pub fn plan(mut self, plan: Plan) -> Self {
		self.plan = plan;
		self
	}

	pub fn os_id(mut self, os_id: u32) -> Self {
		self.os_id = Some(os_id);
		self
	}

	pub fn ipxe_chain_url(mut self, ipxe_chain_url: String) -> Self {
		self.ipxe_chain_url = Some(ipxe_chain_url);
		self
	}

	pub fn iso_id(mut self, iso_id: String) -> Self {
		self.iso_id = Some(iso_id);
		self
	}

	pub fn script_id(mut self, script_id: String) -> Self {
		self.script_id = Some(script_id);
		self
	}

	pub fn snapshot_id(mut self, snapshot_id: String) -> Self {
		self.snapshot_id = Some(snapshot_id);
		self
	}

	pub fn enable_ipv6(mut self, enable_ipv6: bool) -> Self {
		self.enable_ipv6 = Some(enable_ipv6);
		self
	}

	pub fn attach_private_network_deprecated(
		mut self,
		attach_private_network: Vec<String>,
	) -> Self {
		self.attach_private_network_deprecated = Some(attach_private_network);
		self
	}

	pub fn attach_vpc(mut self, attach_vpc: Vec<String>) -> Self {
		self.attach_vpc = Some(attach_vpc);
		self
	}

	pub fn label(mut self, label: String) -> Self {
		self.label = Some(label);
		self
	}

	pub fn sshkey_id(mut self, sshkey_id: Vec<String>) -> Self {
		self.sshkey_id = Some(sshkey_id);
		self
	}

	pub fn backups(mut self, backups: String) -> Self {
		self.backups = Some(backups);
		self
	}

	pub fn app_id(mut self, app_id: u32) -> Self {
		self.app_id = Some(app_id);
		self
	}

	pub fn image_id(mut self, image_id: String) -> Self {
		self.image_id = Some(image_id);
		self
	}

	pub fn user_data(mut self, user_data: String) -> Self {
		self.user_data = Some(user_data);
		self
	}

	pub fn ddos_protection(mut self, ddos_protection: bool) -> Self {
		self.ddos_protection = Some(ddos_protection);
		self
	}

	pub fn activation_email(mut self, activation_email: bool) -> Self {
		self.activation_email = Some(activation_email);
		self
	}

	pub fn hostname(mut self, hostname: String) -> Self {
		self.hostname = Some(hostname);
		self
	}

	pub fn tag(mut self, tag: String) -> Self {
		self.tag = Some(tag);
		self
	}

	pub fn firewall_group_id(mut self, firewall_group_id: String) -> Self {
		self.firewall_group_id = Some(firewall_group_id);
		self
	}

	pub fn reserved_ipv4(mut self, reserved_ipv4: String) -> Self {
		self.reserved_ipv4 = Some(reserved_ipv4);
		self
	}

	pub fn enable_private_network_deprecated(mut self, enable_private_network: bool) -> Self {
		self.enable_private_network_deprecated = Some(enable_private_network);
		self
	}

	pub fn enable_vpc(mut self, enable_vpc: bool) -> Self {
		self.enable_vpc = Some(enable_vpc);
		self
	}

	pub fn tags(mut self, tags: Vec<String>) -> Self {
		self.tags = Some(tags);
		self
	}

	pub async fn create(self, shared_config: SharedConfig) -> Instance {
		shared_config
			.clients
			.vultr()
			.post("https://api.vultr.com/v2/instances")
			.json(&self)
			.send()
			.await
			.unwrap()
			.json::<Instance>()
			.await
			.unwrap()
	}
}

impl Instance {
	pub async fn start(&self, shared_config: SharedConfig) {
		shared_config
			.clients
			.vultr()
			.post("https://api.vultr.com/v2/instances/start")
			.json(&json!({ "instance_ids": vec![self.id.clone()] }))
			.bearer_auth(dotenv!("VULTR_API_KEY"))
			.send()
			.await
			.unwrap();
	}

	pub async fn halt(&self, shared_config: SharedConfig) {
		shared_config
			.clients
			.vultr()
			.post("https://api.vultr.com/v2/instances/halt")
			.json(&json!({ "instance_ids": vec![self.id.clone()] }))
			.bearer_auth(dotenv!("VULTR_API_KEY"))
			.send()
			.await
			.unwrap();
	}

	pub async fn reboot(&self, shared_config: SharedConfig) {
		shared_config
			.clients
			.vultr()
			.post(format!(
				"https://api.vultr.com/v2/instances/{}/reboot",
				self.id
			))
			.bearer_auth(dotenv!("VULTR_API_KEY"))
			.send()
			.await
			.unwrap();
	}

	pub async fn delete(&self, shared_config: SharedConfig) {
		shared_config
			.clients
			.vultr()
			.delete(format!("https://api.vultr.com/v2/instances/{}", self.id))
			.bearer_auth(dotenv!("VULTR_API_KEY"))
			.send()
			.await
			.unwrap();
	}

	pub async fn reinstall(&self, hostname: String, shared_config: SharedConfig) {
		shared_config
			.clients
			.vultr()
			.post(format!(
				"https://api.vultr.com/v2/instances/{}/reinstall",
				self.id
			))
			.json(&json!({
				"hostname": hostname,
			}))
			.bearer_auth(dotenv!("VULTR_API_KEY"))
			.send()
			.await
			.unwrap();
	}

	pub async fn bandwidth(&self, shared_config: SharedConfig) -> HashMap<String, Bandwidth> {
		shared_config
			.clients
			.vultr()
			.get(format!(
				"https://api.vultr.com/v2/instances/{}/bandwidth",
				self.id
			))
			.bearer_auth(dotenv!("VULTR_API_KEY"))
			.send()
			.await
			.unwrap()
			.json::<HashMap<String, Bandwidth>>()
			.await
			.unwrap()
	}
}
