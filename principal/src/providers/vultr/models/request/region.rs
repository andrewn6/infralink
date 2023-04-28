use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::ser::{Serialize, Serializer};
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum Region {
	Asia(Asia),
	Australia(Australia),
	Europe(Europe),
	NorthAmerica(NorthAmerica),
	SouthAmerica(SouthAmerica),
	Africa(Africa),
	Unknown,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Asia {
	Tokyo,
	Osaka,
	Seoul,
	Singapore,
	Mumbai,
	TelAviv,
	Bangalore,
	Delhi,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Australia {
	Sydney,
	Melbourne,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Europe {
	Amsterdam,
	London,
	Frankfurt,
	Paris,
	Warsaw,
	Madrid,
	Stockholm,
}

#[derive(Debug, PartialEq, Clone)]
pub enum NorthAmerica {
	NewJersey,
	Chicago,
	Dallas,
	Seattle,
	LosAngeles,
	Atlanta,
	SiliconValley,
	Toronto,
	Miami,
	MexicoCity,
	Honolulu,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SouthAmerica {
	SaoPaulo,
	Santiago,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Africa {
	Johannesburg,
}

impl ToString for Region {
	fn to_string(&self) -> String {
		match self {
			Region::Asia(city) => {
				match city {
					Asia::Tokyo => "Tokyo".to_string(),
					Asia::Osaka => "Osaka".to_string(),
					Asia::Seoul => "Seoul".to_string(),
					Asia::Singapore => "Singapore".to_string(),
					Asia::Mumbai => "Mumbai".to_string(),
					Asia::TelAviv => "Tel Aviv".to_string(),
					Asia::Bangalore => "Bangalore".to_string(),
					Asia::Delhi => "Delhi".to_string(),
				}
			}
			Region::Australia(city) => {
				match city {
					Australia::Sydney => "Sydney".to_string(),
					Australia::Melbourne => "Melbourne".to_string(),
				}
			}
			Region::Europe(city) => {
				match city {
					Europe::Amsterdam => "Amsterdam".to_string(),
					Europe::London => "London".to_string(),
					Europe::Frankfurt => "Frankfurt".to_string(),
					Europe::Paris => "Paris".to_string(),
					Europe::Warsaw => "Warsaw".to_string(),
					Europe::Madrid => "Madrid".to_string(),
					Europe::Stockholm => "Stockholm".to_string(),
				}
			}
			Region::NorthAmerica(city) => {
				match city {
					NorthAmerica::NewJersey => "New Jersey".to_string(),
					NorthAmerica::Chicago => "Chicago".to_string(),
					NorthAmerica::Dallas => "Dallas".to_string(),
					NorthAmerica::Seattle => "Seattle".to_string(),
					NorthAmerica::LosAngeles => "Los Angeles".to_string(),
					NorthAmerica::Atlanta => "Atlanta".to_string(),
					NorthAmerica::SiliconValley => "Silicon Valley".to_string(),
					NorthAmerica::Toronto => "Toronto".to_string(),
					NorthAmerica::Miami => "Miami".to_string(),
					NorthAmerica::MexicoCity => "Mexico City".to_string(),
					NorthAmerica::Honolulu => "Honolulu".to_string(),
				}
			}
			Region::SouthAmerica(city) => {
				match city {
					SouthAmerica::SaoPaulo => "Sao Paulo".to_string(),
					SouthAmerica::Santiago => "Santiago".to_string(),
				}
			}
			Region::Africa(city) => {
				match city {
					Africa::Johannesburg => "Johannesburg".to_string(),
				}
			}
			Region::Unknown => "Unknown".to_string(),
		}
	}
}

impl Region {
	fn code(&self) -> String {
		match self {
			Region::Asia(city) => {
				match city {
					Asia::Tokyo => "nrt".to_string(),
					Asia::Osaka => "itm".to_string(),
					Asia::Seoul => "icn".to_string(),
					Asia::Singapore => "sgp".to_string(),
					Asia::Mumbai => "bom".to_string(),
					Asia::TelAviv => "tlv".to_string(),
					Asia::Bangalore => "blr".to_string(),
					Asia::Delhi => "del".to_string(),
				}
			}
			Region::Australia(city) => {
				match city {
					Australia::Sydney => "syd".to_string(),
					Australia::Melbourne => "mel".to_string(),
				}
			}
			Region::Europe(city) => {
				match city {
					Europe::Amsterdam => "ams".to_string(),
					Europe::London => "lhr".to_string(),
					Europe::Frankfurt => "fra".to_string(),
					Europe::Paris => "cdg".to_string(),
					Europe::Warsaw => "waw".to_string(),
					Europe::Madrid => "mad".to_string(),
					Europe::Stockholm => "sto".to_string(),
				}
			}
			Region::NorthAmerica(city) => {
				match city {
					NorthAmerica::NewJersey => "ewr".to_string(),
					NorthAmerica::Chicago => "ord".to_string(),
					NorthAmerica::Dallas => "dfw".to_string(),
					NorthAmerica::Seattle => "sea".to_string(),
					NorthAmerica::LosAngeles => "lax".to_string(),
					NorthAmerica::Atlanta => "atl".to_string(),
					NorthAmerica::SiliconValley => "sjc".to_string(),
					NorthAmerica::Toronto => "yto".to_string(),
					NorthAmerica::Miami => "mia".to_string(),
					NorthAmerica::MexicoCity => "mex".to_string(),
					NorthAmerica::Honolulu => "hnl".to_string(),
				}
			}
			Region::SouthAmerica(city) => {
				match city {
					SouthAmerica::SaoPaulo => "sao".to_string(),
					SouthAmerica::Santiago => "scl".to_string(),
				}
			}
			Region::Africa(city) => {
				match city {
					Africa::Johannesburg => "jnb".to_string(),
				}
			}
			Region::Unknown => "Unknown".to_string(),
		}
	}

	pub fn from_code(code: &str) -> Result<Self, &'static str> {
		match code {
			"nrt" => Ok(Region::Asia(Asia::Tokyo)),
			"itm" => Ok(Region::Asia(Asia::Osaka)),
			"icn" => Ok(Region::Asia(Asia::Seoul)),
			"sgp" => Ok(Region::Asia(Asia::Singapore)),
			"bom" => Ok(Region::Asia(Asia::Mumbai)),
			"tlv" => Ok(Region::Asia(Asia::TelAviv)),
			"blr" => Ok(Region::Asia(Asia::Bangalore)),
			"del" => Ok(Region::Asia(Asia::Delhi)),
			"syd" => Ok(Region::Australia(Australia::Sydney)),
			"mel" => Ok(Region::Australia(Australia::Melbourne)),
			"ams" => Ok(Region::Europe(Europe::Amsterdam)),
			"lhr" => Ok(Region::Europe(Europe::London)),
			"fra" => Ok(Region::Europe(Europe::Frankfurt)),
			"cdg" => Ok(Region::Europe(Europe::Paris)),
			"waw" => Ok(Region::Europe(Europe::Warsaw)),
			"mad" => Ok(Region::Europe(Europe::Madrid)),
			"sto" => Ok(Region::Europe(Europe::Stockholm)),
			"ewr" => Ok(Region::NorthAmerica(NorthAmerica::NewJersey)),
			"ord" => Ok(Region::NorthAmerica(NorthAmerica::Chicago)),
			"dfw" => Ok(Region::NorthAmerica(NorthAmerica::Dallas)),
			"sea" => Ok(Region::NorthAmerica(NorthAmerica::Seattle)),
			"lax" => Ok(Region::NorthAmerica(NorthAmerica::LosAngeles)),
			"atl" => Ok(Region::NorthAmerica(NorthAmerica::Atlanta)),
			"sjc" => Ok(Region::NorthAmerica(NorthAmerica::SiliconValley)),
			"yto" => Ok(Region::NorthAmerica(NorthAmerica::Toronto)),
			"mia" => Ok(Region::NorthAmerica(NorthAmerica::Miami)),
			"mex" => Ok(Region::NorthAmerica(NorthAmerica::MexicoCity)),
			"hnl" => Ok(Region::NorthAmerica(NorthAmerica::Honolulu)),
			"sao" => Ok(Region::SouthAmerica(SouthAmerica::SaoPaulo)),
			"scl" => Ok(Region::SouthAmerica(SouthAmerica::Santiago)),
			"jnb" => Ok(Region::Africa(Africa::Johannesburg)),
			"unknown" => Ok(Region::Unknown),
			_ => Err("Unknown region code"),
		}
	}
}

impl Serialize for Region {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(&self.code())
	}
}

struct RegionVisitor;

impl<'de> Visitor<'de> for RegionVisitor {
	type Value = Region;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		formatter.write_str("a region code string")
	}

	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: de::Error,
	{
		Region::from_code(v).map_err(|err| de::Error::custom(err))
	}
}

impl<'de> Deserialize<'de> for Region {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_str(RegionVisitor)
	}
}
