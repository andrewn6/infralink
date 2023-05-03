use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::shared_config::SharedConfig;

use super::region::Region;

#[derive(Serialize, Deserialize, Debug)]
pub enum Architecture {
	X86,
	Arm,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CpuType {
	Shared,
	Dedicated,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum FirewallStatus {
	Applied,
	Pending,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ImageStatus {
	Available,
	Creating,
	Unavailable,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ImageType {
	System,
	App,
	Snapshot,
	Backup,
	Temporary,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum InstanceStatus {
	Running,
	Initializing,
	Starting,
	Stopping,
	Off,
	Deleting,
	Migrating,
	Rebuilding,
	Unknown,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum IsoType {
	Public,
	Private,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PlacementGroupType {
	Spread,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum StorageType {
	Local,
	Network,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreatedFromObject {
	id: u64,
	name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DataCenter {
	id: u64,
	name: String,
	description: String,
	location: LocationObject,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DnsPTR {
	ip: String,
	dns_ptr: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FirewallInstance {
	pub id: u64,
	pub status: FirewallStatus,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImageObject {
	id: u32,
	name: String,
	bound_to: Option<u32>,
	created: String,
	created_from: CreatedFromObject,
	deleted: Option<String>,
	deprecated: Option<String>,
	description: String,
	disk_size: u64,
	image_size: u64,
	os_flavor: String,
	os_version: Option<String>,
	protection: ProtectionObject,
	rapid_deploy: Option<bool>,
	status: ImageStatus,
	r#type: ImageType,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IPAddress {
	pub id: u64,
	pub blocked: bool,
	pub dns_ptr: Option<Vec<DnsPTR>>,
	pub ip: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Iso {
	id: u64,
	name: String,
	architecture: Option<Architecture>,
	deprecated: String,
	description: String,
	r#type: IsoType,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LocationObject {
	id: u64,
	name: String,
	city: String,
	country: String,
	description: String,
	latitude: f32,
	longitude: f32,
	network_zone: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlacementGroup {
	name: String,
	r#type: PlacementGroupType,
	id: u64,
	created: String,
	labels: HashMap<String, String>,
	servers: Vec<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pricing {
	location: String,
	price_hourly: String,
	price_monthly: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PricingModel {
	gross: String,
	net: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PrivateNet {
	pub alias_ips: Vec<String>,
	pub ip: String,
	pub mac_address: String,
	pub network: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProtectionObject {
	delete: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProtectionObjectInstance {
	delete: bool,
	rebuild: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PublicNetInstance {
	firewalls: Vec<FirewallInstance>,
	floating_ips: Vec<u64>,
	ipv4: IPAddress,
	ipv6: IPAddress,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerType {
	id: u64,
	name: String,
	cores: u64,
	cpu_type: CpuType,
	deprecated: bool,
	disk: u64,
	memory: u64,
	storage_type: StorageType,
	prices: Vec<Pricing>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Instance {
	pub id: u64,
	pub name: String,
	pub backup_window: Option<String>,
	pub created: String,
	pub datacenter: DataCenter,
	pub image: ImageObject,
	pub included_traffic: u64,
	pub ingoing_traffic: u64,
	pub outgoing_traffic: Option<u64>,
	pub iso: Option<Iso>,
	pub labels: HashMap<String, String>,
	pub load_balancers: Vec<u64>,
	pub locked: bool,
	pub placement_group: Option<PlacementGroup>,
	pub primary_disk_size: u64,
	pub private_net: Vec<PrivateNet>,
	pub protection: ProtectionObjectInstance,
	pub public_net: PublicNetInstance,
	pub rescue_enabled: bool,
	pub server_type: ServerType,
	pub status: InstanceStatus,
	pub volumes: Vec<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PublicNet {
	pub enable_ipv4: Option<bool>,
	pub enable_ipv6: Option<bool>,
	pub ipv4: Option<u64>,
	pub ipv6: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum InstanceType {
	Sharedx86(SharedX86),       // Shared x86 Instances
	DedicatedX86(DedicatedX86), // Dedicated x86 Instances
	SharedArm(SharedArm),       // Shared ARM Instances
	Unknown,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SharedX86 {
	CX11,  // 1vCPU 2GB RAM (Intel)
	CPX11, // 2vCPU 2GB RAM (AMD)
	CX21,  // 2vCPU 4GB RAM (Intel)
	CPX21, // 3vCPU 4GB RAM (AMD)
	CX31,  // 2vCPU 8GB RAM (Intel)
	CPX31, // 4vCPU 8GB RAM (AMD)
	CX41,  // 4vCPU 16GB RAM (Intel)
	CPX41, // 8vCPU 16GB RAM (AMD)
	CX51,  // 8vCPU 32GB RAM (Intel)
	CPX51, // 16vCPU 32GB RAM (AMD)
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SharedArm {
	CAX11, // 2vCPU 4GB RAM
	CAX21, // 4vCPU 8GB RAM
	CAX31, // 8vCPU 16GB RAM
	CAX41, // 16vCPU 32GB RAM
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DedicatedX86 {
	CCX11, // 2vCPU 8GB RAM (Intel)
	CCX12, // 2vCPU 8GB RAM (AMD)
	CCX21, // 4vCPU 16GB RAM (Intel)
	CCX22, // 4vCPU 16GB RAM (AMD)
	CCX31, // 8vCPU 32GB RAM (Intel)
	CCX32, // 8vCPU 32GB RAM (AMD)
	CCX41, // 16vCPU 64GB RAM (Intel)
	CCX42, // 16vCPU 64GB RAM (AMD)
	CCX51, // 32vCPU 128GB RAM (Intel)
	CCX52, // 32vCPU 128GB RAM (AMD)
	CCX62, // 48vCPU 192GB RAM (AMD)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InstanceBuilder {
	pub name: String,
	pub automount: Option<bool>,
	pub datacenter: Option<String>,
	pub firewalls: Option<Vec<Firewall>>,
	pub image: String,
	pub labels: HashMap<String, String>,
	pub location: Option<Region>,
	pub networks: Option<Vec<u64>>,
	pub placement_group: Option<u64>,
	pub public_net: Option<PublicNet>,
	pub server_type: InstanceType,
	pub ssh_keys: Option<Vec<String>>,
	pub start_after_create: Option<bool>,
	pub user_data: String,
	pub volumes: Vec<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Firewall {
	id: u64,
}

impl InstanceBuilder {
	pub fn new() -> Self {
		InstanceBuilder {
			automount: None,
			datacenter: None,
			firewalls: None,
			image: String::new(),
			labels: HashMap::new(),
			location: None,
			name: String::new(),
			networks: None,
			placement_group: None,
			public_net: None,
			server_type: InstanceType::Unknown,
			ssh_keys: None,
			start_after_create: None,
			user_data: String::new(),
			volumes: Vec::new(),
		}
	}

	pub fn automount(mut self, automount: bool) -> Self {
		self.automount = Some(automount);
		self
	}

	pub fn datacenter(mut self, datacenter: String) -> Self {
		self.datacenter = Some(datacenter);
		self
	}

	pub fn firewalls(mut self, firewalls: Vec<Firewall>) -> Self {
		self.firewalls = Some(firewalls);
		self
	}

	pub fn image(mut self, image: String) -> Self {
		self.image = image;
		self
	}

	pub fn labels(mut self, labels: HashMap<String, String>) -> Self {
		self.labels = labels;
		self
	}

	pub fn location(mut self, location: Region) -> Self {
		self.location = Some(location);
		self
	}

	pub fn name(mut self, name: String) -> Self {
		self.name = name;
		self
	}

	pub fn networks(mut self, networks: Vec<u64>) -> Self {
		self.networks = Some(networks);
		self
	}

	pub fn placement_group(mut self, placement_group: u64) -> Self {
		self.placement_group = Some(placement_group);
		self
	}

	pub fn public_net(mut self, public_net: PublicNet) -> Self {
		self.public_net = Some(public_net);
		self
	}

	pub fn server_type(mut self, server_type: InstanceType) -> Self {
		self.server_type = server_type;
		self
	}

	pub fn ssh_keys(mut self, ssh_keys: Vec<String>) -> Self {
		self.ssh_keys = Some(ssh_keys);
		self
	}

	pub fn start_after_create(mut self, start_after_create: bool) -> Self {
		self.start_after_create = Some(start_after_create);
		self
	}

	pub fn user_data(mut self, user_data: String) -> Self {
		self.user_data = user_data;
		self
	}

	pub fn volumes(mut self, volumes: Vec<u64>) -> Self {
		self.volumes = volumes;
		self
	}

	pub async fn build(self, shared_config: SharedConfig) -> Instance {
		shared_config
			.clients
			.hetzner()
			.post("https://api.hetzner.cloud/v1/servers")
			.json(&self)
			.send()
			.await
			.unwrap()
			.json::<Instance>()
			.await
			.unwrap()
	}
}
