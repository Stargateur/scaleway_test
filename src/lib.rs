use std::sync::LazyLock;

use tokio::sync::RwLock;

pub mod dhcp;
pub mod dns;
pub mod ipam;
pub mod vpc;

pub const MAX_MTU: usize = 576;

static VPC: LazyLock<RwLock<vpc::VpcApiMock>> = LazyLock::new(|| {
    vpc::VpcApiMock::new().into()
});

static IPAM: LazyLock<RwLock<ipam::IpamApiMock>> = LazyLock::new(|| {
    ipam::IpamApiMock::new().into()
});