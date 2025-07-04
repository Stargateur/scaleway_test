use std::{
  collections::HashMap,
  convert::AsRef,
  net::Ipv4Addr,
  sync::Arc,
};

use ipnet::Ipv4Net;

pub trait Vpc {
  fn find_pn_by_subnet(&self, subnet_id: &SubnetID) -> Option<&PrivateNetwork>;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubnetID(Arc<str>);

impl SubnetID {
  pub fn new(id: impl Into<Arc<str>>) -> Self {
    Self(id.into())
  }
}

impl AsRef<str> for SubnetID {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

#[derive(Debug, Clone)]
pub struct Subnet {
  pub id: SubnetID,
  pub cidr: Ipv4Net,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PnId(Arc<str>);

impl PnId {
  pub fn new(id: impl Into<Arc<str>>) -> Self {
    Self(id.into())
  }
}

impl AsRef<str> for PnId {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

#[derive(Debug, Clone)]
pub struct PrivateNetwork {
  pub id: PnId,
  pub name: String,
  pub vni: u32,
  pub subnets: HashMap<SubnetID, Subnet>,
}

pub struct VpcApiMock {
  pub pns: HashMap<PnId, PrivateNetwork>,
}

impl VpcApiMock {
  pub fn new() -> Self {
    let subnet = Subnet {
      id: SubnetID::new("subnet-1"),
      cidr: Ipv4Net::new(Ipv4Addr::new(192, 168, 1, 0), 24).unwrap(),
    };
    let mut subnets = HashMap::new();
    subnets.insert(subnet.id.clone(), subnet);

    let pn = PrivateNetwork {
      id: PnId::new("pn-1"),
      name: "backend".to_string(),
      vni: 42,
      subnets,
    };
    let mut pns = HashMap::new();
    pns.insert(pn.id.clone(), pn);

    Self { pns }
  }
}

impl Vpc for VpcApiMock {
  fn find_pn_by_subnet(&self, subnet_id: &SubnetID) -> Option<&PrivateNetwork> {
    self
      .pns
      .values()
      .find(|pn| pn.subnets.contains_key(&subnet_id))
  }
}
