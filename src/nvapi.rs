//! # NVIDIA API
//! This module contains actions related to the NVIDIA API. Not to be confused with NVIDIA's driver api.
//! Reference: <https://github.com/fyr77/EnvyUpdate/wiki/Nvidia-API>

use std::error::Error;

use serde::Deserialize;

pub struct XmlGpuEntry {
	pub name: String, // e.g. "GeForce RTX 3090 Ti"
	pub series: u16, // e.g. "120", the ParentID in the XML file
	pub id: u16, // e.g. "985"
}

#[derive(Debug, Deserialize)]
pub struct LookupValueSearch {
	#[serde(rename="LookupValues")]
	pub lookupvalues: LookupValues,
}

#[derive(Debug, Deserialize)]
pub struct LookupValues {
	#[serde(rename="LookupValue")]
	pub lookupvalue: Vec<LookupValue>,
}

#[derive(Debug, Deserialize)]
pub struct LookupValue {
	#[serde(rename="ParentID")]
	pub parentid: u16,
	#[serde(rename="Name")]
	pub name: Name,
	#[serde(rename="Value")]
	pub value: Value,
}

#[derive(Debug, Deserialize)]
pub struct Name {
	#[serde(rename = "$value")]
	pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct Value {
	#[serde(rename = "$value")]
	pub value: u16,
}

pub async fn get_gpu_list() -> Result<Vec<XmlGpuEntry>, Box<dyn Error>> {
	let xml = reqwest::get("https://www.nvidia.com/Download/API/lookupValueSearch.aspx?TypeID=3");
	let deserialized: LookupValueSearch = quick_xml::de::from_str(&xml.await?.text().await?)?;
	println!("{:#?}", deserialized);

	let mut gpu_entries: Vec<XmlGpuEntry> = Vec::new();
	for lookupvalue in deserialized.lookupvalues.lookupvalue.iter() {
		let gpu_entry = XmlGpuEntry {
			name: lookupvalue.name.value.clone(),
			series: lookupvalue.parentid,
			id: lookupvalue.value.value,
		};
		gpu_entries.push(gpu_entry);
	}
	Ok(gpu_entries)
}