use std::{
  collections::{
    HashMap,
    hash_map::Entry,
  },
  net::Ipv4Addr,
  time::Duration,
};

use mac_address::MacAddress;
use snafu::Snafu;
use tokio::time::Instant;

use crate::vpc::{
  SubnetID,
  VpcApiMock,
};

pub(crate) const DEFAULT_DURATION: Duration = Duration::from_secs(60 * 60 * 24 * 42); // 42 days

#[derive(Snafu, Debug)]
pub(crate) enum Error {
  #[snafu(display("IP not found for MAC {}", mac))]
  IpNotFound { mac: MacAddress },
}

pub trait Ipam {
  fn assign_ip(
    &mut self, mac: MacAddress, vpc_api: &VpcApiMock, duration: Duration,
  ) -> Option<Ipv4Addr>;
  fn get_ip_by_mac(&self, mac: MacAddress) -> Option<&Lease>;
  fn get_mut_ip_by_mac(&mut self, mac: MacAddress) -> Option<&mut Lease>;
  fn get_ip_by_name(&self, name: &str) -> Option<&Lease>;
  fn renew_lease(&mut self, mac: MacAddress) -> Result<(), Error>;
}

impl Ipam for IpamApiMock {
  fn assign_ip(
    &mut self, mac: MacAddress, vpc: &VpcApiMock, duration: Duration,
  ) -> Option<Ipv4Addr> {
    self.check_duration_leaves();

    if let Some(existing) = self.get_ip_by_mac(mac) {
      return Some(existing.ip.clone());
    }

    // do better then first
    let pn = vpc.pns.values().next()?;
    let subnet = pn.subnets.values().next()?;
    // do something with subnet

    for ip in (1..255).map(|i| Ipv4Addr::new(192, 168, 50, i)) {
      if let Entry::Vacant(vacant) = self.assignments.entry(ip) {
        let resource = Resource {
          id: format!("res-{}", mac),
          name: format!("machine-{}", mac),
          mac,
        };
        let assignment = Lease {
          ip,
          subnet_id: subnet.id.clone(),
          resource,
          duration,
          updated: Instant::now(),
        };
        vacant.insert(assignment);
        return Some(ip);
      }
    }

    // error ?
    None
  }

  fn renew_lease(&mut self, mac: MacAddress) -> Result<(), Error> {
    return if let Some(lease) = self.get_mut_ip_by_mac(mac) {
      lease.duration = DEFAULT_DURATION;
      lease.updated = Instant::now();

      Ok(())
    } else {
      Err(Error::IpNotFound { mac })
    };
  }

  fn get_ip_by_mac(&self, mac: MacAddress) -> Option<&Lease> {
    self.assignments.values().find(|a| a.resource.mac == mac)
  }

  fn get_mut_ip_by_mac(&mut self, mac: MacAddress) -> Option<&mut Lease> {
    self
      .assignments
      .values_mut()
      .find(|a| a.resource.mac == mac)
  }

  fn get_ip_by_name(&self, name: &str) -> Option<&Lease> {
    self.assignments.values().find(|a| a.resource.name == name)
  }
}

pub struct IpamApiMock {
  assignments: HashMap<Ipv4Addr, Lease>,
}

impl IpamApiMock {
  pub fn new() -> Self {
    Self {
      assignments: HashMap::new(),
    }
  }

  fn check_duration_leaves(&mut self) {
    let now = Instant::now();
    self.assignments.retain(|_, lease| {
      now.duration_since(lease.updated) < lease.duration && lease.duration != Duration::from_secs(0)
    });
  }
}

#[derive(Debug, Clone)]
pub struct Lease {
  pub ip: Ipv4Addr,
  pub subnet_id: SubnetID,
  pub duration: Duration,
  pub updated: Instant,
  pub resource: Resource,
}

#[derive(Debug, Clone)]
pub struct Resource {
  pub id: String,
  pub name: String,
  pub mac: MacAddress,
}
