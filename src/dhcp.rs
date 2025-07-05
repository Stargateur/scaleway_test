use std::{
  net::{
    Ipv4Addr,
    SocketAddrV4,
  },
  str::FromStr,
  time::Duration,
};

use dhcproto::{
  Decodable,
  Decoder,
  Encodable,
  Encoder,
  v4::{
    DhcpOption,
    Message,
    MessageType,
    Opcode,
  },
};
use mac_address::MacAddress;
use snafu::{
  OptionExt,
  ResultExt,
  Snafu,
};
use tokio::net::UdpSocket;
use tracing::{
  error,
  info,
};

use crate::{
  ipam::{
    self,
    Ipam,
    IpamApiMock,
    Resource,
  },
  vpc::{
    Vpc,
    VpcApiMock,
  },
};

#[derive(Debug, Snafu)]
pub enum Error {
  #[snafu(display("Socket error: {}", source))]
  Socket { source: std::io::Error },
  #[snafu(display("Decoding error: {}", source))]
  Decode {
    source: dhcproto::error::DecodeError,
  },
  #[snafu(display("Encoding error: {}", source))]
  Encode {
    source: dhcproto::error::EncodeError,
  },
  #[snafu(display("No Ip available"))]
  NoIpAvailable {},
}

pub struct DhcpServer {
  socket: UdpSocket,
  mac: MacAddress,
  ip: Ipv4Addr,
}

impl DhcpServer {
  pub async fn new() -> Result<DhcpServer, Error> {
    let socket = UdpSocket::bind("0.0.0.0:67").await.context(SocketSnafu)?;
    socket.set_broadcast(true).context(SocketSnafu)?;
    info!("DHCP server listening on UDP/67");

    let mut ipam = crate::IPAM.write().await;
    let mac = mac_address::get_mac_address()
      .expect("Failed to get MAC address")
      .expect("No MAC address found");
    let vpc = crate::VPC.read().await;

    // we want DHCP server to have first IP
    let ip = ipam
      .assign_ip(mac, &vpc, Duration::from_secs(0))
      .expect("No IP available and DHCP need one");

    Ok(DhcpServer { socket, mac, ip })
  }

  async fn run(&self) -> Result<(), Error> {
    let mut buf = std::vec![0u8; crate::MAX_MTU];

    loop {
      let (n, addr) = self.socket.recv_from(&mut buf).await.context(SocketSnafu)?;
      info!("Received {} bytes from {}", n, addr);

      let msg = Message::decode(&mut Decoder::new(&buf[..n])).context(DecodeSnafu)?;
      info!("Received DHCP message: {:?}", msg);

      if msg.opcode() != Opcode::BootRequest {
        info!("Ignoring non-request DHCP message: {:?}", msg);
      } else {
        let response = self.request(msg).await;
      }
    }
  }

  async fn request(&self, msg: Message) -> Result<Option<Message>, Error> {
    match msg.opts().msg_type() {
      Some(dhcproto::v4::MessageType::Discover) => {
        let mac = MacAddress::new(msg.chaddr().try_into().expect("Invalid MAC address"));

        let mut ipam = crate::IPAM.write().await;
        let vpc = crate::VPC.read().await;
        let ipv4_addr = ipam
          .assign_ip(mac, &vpc, Duration::from_secs(30))
          .context(NoIpAvailableSnafu)?;
        info!("Assigned IP address: {}", ipv4_addr);
      }
      Some(dhcproto::v4::MessageType::Request) => info!("Received DHCP Request"),
      _ => error!("Received unknown or unsupported DHCP message type"),
    }

    todo!()
  }

  fn build_offer(&self, request: &Message, ip: Ipv4Addr, subnet: &str) -> Message {
    let mut offer = Message::default();
    offer.set_opcode(Opcode::BootReply);
    offer.set_htype(request.htype());
    offer.set_xid(request.xid());
    offer.set_yiaddr(ip);
    offer.set_chaddr(request.chaddr());

    let mut opts = offer.opts_mut();
    opts.insert(DhcpOption::MessageType(MessageType::Offer));
    opts.insert(DhcpOption::ServerIdentifier(self.ip));
    opts.insert(DhcpOption::AddressLeaseTime(
      ipam::DEFAULT_DURATION.as_secs() as u32,
    ));

    opts.insert(DhcpOption::SubnetMask("255.255.255.0".parse().unwrap()));
    // need router ? no idea what to do

    offer
  }
}

pub fn handle_dhcp_request(
  mac: &str, vpc_api: &VpcApiMock, ipam_api: &mut IpamApiMock,
) -> Option<Ipv4Addr> {
  let mac = MacAddress::from_str(mac).expect("Invalid MAC address");
  ipam_api.assign_ip(mac, vpc_api, Duration::from_secs(30))
}
