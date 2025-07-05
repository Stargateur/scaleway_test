use std::{net::Ipv4Addr, ops::Not};

use hickory_proto::op::{Message, OpCode, ResponseCode};
use snafu::{OptionExt, ResultExt, Snafu};
use tokio::net::UdpSocket;

use crate::{
  ipam::{Ipam, IpamApiMock},
  vpc::{PrivateNetwork, Vpc, VpcApiMock},
};

// pub fn resolve_dns(name: &str, pn: &str, ipam_api: impl Ipam) ->
// Option<String> {   let fqdn = format!("{}.{}.internal", name, pn);
//   ipam_api
//     .get_ip_by_name(name)
//     .values()
//     .find(|a| format!("{}.{}.internal", a.resource.name, pn) == fqdn)
//     .map(|a| a.ip.clone())
// }

#[derive(Debug, Snafu)]
pub enum Error {
  #[snafu(display("Socket error: {}", source))]
  Socket { source: std::io::Error },
  #[snafu(display("Not implemented"))]
  NotImplemented,
  #[snafu(display("Usage error"))]
  Usage,
  #[snafu(display("Not found"))]
  NotFound,
  #[snafu(display("Proto: {}", source))]
  Proto { source: hickory_proto::error::Error },
}

pub struct DnsServer {
  socket: UdpSocket,
}

impl DnsServer {
  pub async fn new() -> Result<DnsServer, Error> {
    let socket = UdpSocket::bind("0.0.0.0:53").await.context(SocketSnafu)?;

    Ok(DnsServer { socket })
  }

  pub async fn run(&self) -> Result<(), Error> {
    let mut buf = std::vec![0u8; crate::MAX_MTU];

    loop {
      let (len, peer) = self.socket.recv_from(&mut buf).await.context(SocketSnafu)?;
      let msg = hickory_proto::op::Message::from_vec(&buf[..len]).context(ProtoSnafu)?;

      match self.handle_request(msg).await {
        Ok(resp) => {
          
        }
        Err(e) => {
          tracing::error!("Error handling DNS request: {}", e);
          let mut resp = Message::new();
          resp.set_id(msg.id());
          resp.set_response_code(ResponseCode::ServFail);
          self.socket.send_to(&resp, peer).await?;
        }
      }

    }
  }

  pub async fn handle_request(&self, msg: Message) -> Result<Message, Error> {
    match msg.op_code() {
      OpCode::Query => {
        let mut response = Message::new();
        response.set_id(msg.id());

        for query in msg.queries() {
          let mut iter = query.name().iter().flat_map(|s| str::from_utf8(s).ok());
          let name = iter.next().context(UsageSnafu)?;
          let pn_name = iter.next().context(UsageSnafu)?;
          let internal = iter.next().context(UsageSnafu)?;
          if internal != "internal" || iter.next().is_some() {
            return Err(Error::Usage);
          }

          let vpc = crate::VPC.read().await;
          let ipam = crate::IPAM.read().await;
          if let Some(ip) = resolve_dns(name, pn_name, &vpc, &ipam) {
            response.add_answer(hickory_proto::rr::Record::from_rdata(
              query.name().clone(),
              1337,
              hickory_proto::rr::RData::A(ip.into()),
            ));
          } else {
            return Err(Error::NotFound);
          }
        }

        Ok(response)
      }
      _ => Err(Error::NotImplemented),
    }
  }
}

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
