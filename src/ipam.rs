use std::{
  collections::{
    HashMap,
    hash_map::Entry,
  },
  net::Ipv4Addr,
};

use crate::vpc::{
  SubnetID,
  VpcApiMock,
};

pub trait Ipam {
  fn assign_ip(&mut self, mac: &str, vpc_api: &VpcApiMock) -> Option<Ipv4Addr>;
  fn get_ip_by_mac(&self, mac: &str) -> Option<&IpAssignment>;
  fn get_ip_by_name(&self, name: &str) -> Option<&IpAssignment>;
}

impl Ipam for IpamApiMock {
  fn assign_ip(&mut self, mac: &str, vpc_api: &VpcApiMock) -> Option<Ipv4Addr> {
    if let Some(existing) = self.get_ip_by_mac(mac) {
      return Some(existing.ip.clone());
    }

    // do better then first
    let pn = vpc_api.pns.values().next()?;
    let subnet = pn.subnets.values().next()?;

    for ip in (1..255).map(|i| Ipv4Addr::new(192, 168, 1, i)) {
      if let Entry::Vacant(vacant) = self.assignments.entry(ip) {
        let resource = Resource {
          id: format!("res-{}", mac),
          name: format!("machine-{}", mac),
          mac: mac.to_string(),
        };
        let assignment = IpAssignment {
          ip,
          subnet_id: subnet.id.clone(),
          resource,
        };
        vacant.insert(assignment);
        return Some(ip);
      }
    }

    // error ?
    None
  }

  fn get_ip_by_mac(&self, mac: &str) -> Option<&IpAssignment> {
    self.assignments.values().find(|a| a.resource.mac == mac)
  }

  fn get_ip_by_name(&self, name: &str) -> Option<&IpAssignment> {
    self.assignments.values().find(|a| a.resource.name == name)
  }
}

pub struct IpamApiMock {
  assignments: HashMap<Ipv4Addr, IpAssignment>,
}

impl IpamApiMock {
  pub fn new() -> Self {
    Self {
      assignments: HashMap::new(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct IpAssignment {
  pub ip: Ipv4Addr,
  pub subnet_id: SubnetID,
  pub resource: Resource,
}

#[derive(Debug, Clone)]
pub struct Resource {
  pub id: String,
  pub name: String,
  pub mac: String,
}
