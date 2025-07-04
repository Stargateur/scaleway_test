use std::{
  collections::HashMap,
  convert::AsRef,
  net::Ipv4Addr,
  sync::Arc,
};

use ipnet::Ipv4Net;
use snafu::{
  OptionExt,
  ResultExt,
  Snafu,
};

#[derive(Snafu, Debug)]
pub enum Error {
  #[snafu(display("Subnet not found: {:?}", subnet_id))]
  SubnetNotFound { subnet_id: SubnetID },
  #[snafu(display("Private Network not found: {:?}", pn_id))]
  PnNotFound { pn_id: PnId },
  #[snafu(display("Subnet {:?} overlaps with existing subnet {:?} in PN {:?}", subnet_id, other_id, pn_id))]
  SubnetOverlap {
    subnet_id: SubnetID,
    other_id: SubnetID,
    pn_id: PnId,
  },
}

pub trait Vpc {
  fn find_pn_by_subnet(&self, subnet_id: &SubnetID) -> Option<&PrivateNetwork>;
  fn add_subnet(&mut self, pn_id: PnId, subnet: Subnet) -> Result<(), Error>;
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

  fn check_if_subnet_overlaps(
    pn: &PrivateNetwork,
    subnet: &Subnet,
  ) -> Result<(), Error> {
    pn.subnets
      .values()
      .find(|s| s.cidr.contains(&subnet.cidr) || subnet.cidr.contains(&s.cidr))
      .map_or(Ok(()), |other_id| {
        Err(Error::SubnetOverlap {
          subnet_id: subnet.id.clone(),
          other_id: other_id.id.clone(),
          pn_id: pn.id.clone(),
        })
      })
  }
}

impl Vpc for VpcApiMock {
  fn find_pn_by_subnet(&self, subnet_id: &SubnetID) -> Option<&PrivateNetwork> {
    self
      .pns
      .values()
      .find(|pn| pn.subnets.contains_key(&subnet_id))
  }

  fn add_subnet(&mut self, pn_id: PnId, subnet: Subnet) -> Result<(), Error> {
    let pn = self.pns.get_mut(&pn_id).with_context(|| PnNotFoundSnafu {
      pn_id: pn_id.clone(),
    })?;

    Self::check_if_subnet_overlaps(pn, &subnet)?;

    pn.subnets.insert(subnet.id.clone(), subnet);

    Ok(())
  }
}
