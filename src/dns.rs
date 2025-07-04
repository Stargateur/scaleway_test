use std::net::Ipv4Addr;

use crate::{
  ipam::{
    Ipam,
    IpamApiMock,
  },
  vpc::{
    PrivateNetwork,
    Vpc,
    VpcApiMock,
  },
};

// pub fn resolve_dns(name: &str, pn: &str, ipam_api: impl Ipam) ->
// Option<String> {   let fqdn = format!("{}.{}.internal", name, pn);
//   ipam_api
//     .get_ip_by_name(name)
//     .values()
//     .find(|a| format!("{}.{}.internal", a.resource.name, pn) == fqdn)
//     .map(|a| a.ip.clone())
// }

pub fn resolve_dns(
  name: &str, pn_name: &str, vpc_api: &VpcApiMock, ipam_api: &IpamApiMock,
) -> Option<Ipv4Addr> {
  // Recherche sur toutes les attributions IP
  let rss = ipam_api.get_ip_by_name(name)?;
  let pn = vpc_api.find_pn_by_subnet(&rss.subnet_id)?;

  if pn.name == pn_name {
    Some(rss.ip.clone())
  } else {
    None
  }
}
