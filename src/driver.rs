use embassy_net::driver::{Driver, RxToken};
use crate::patch;

pub struct FirewallDevice<D: Driver> {
    inner: D,
}

impl<D: Driver> FirewallDevice<D> {
    pub fn new(device: D) -> Self {
        Self { inner: device }
    }
    
    fn check_packet(&self, data: &[u8]) -> bool {
        let result = patch::xdp(data);
        // 根据你的 XDP 程序返回值决定是否接受数据包
        // 当前返回 42，你可以定义自己的协议
        result == 42  // 或者 result != 0 表示接受
    }
}

impl<D: Driver> Driver for FirewallDevice<D> {
    type RxToken<'a> = FirewallRxToken<D::RxToken<'a>, D> where Self: 'a;
    type TxToken<'a> = D::TxToken<'a> where Self: 'a;

    fn receive(&mut self, cx: &mut core::task::Context) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let firewall_ptr = self as *const _;
        self.inner.receive(cx).map(|(rx, tx)| {
            (FirewallRxToken { inner: rx, firewall: firewall_ptr }, tx)
        })
    }

    fn transmit(&mut self, cx: &mut core::task::Context) -> Option<Self::TxToken<'_>> {
        self.inner.transmit(cx)
    }

    fn link_state(&mut self, cx: &mut core::task::Context) -> embassy_net::driver::LinkState {
        self.inner.link_state(cx)
    }

    fn capabilities(&self) -> embassy_net::driver::Capabilities {
        self.inner.capabilities()
    }

    fn hardware_address(&self) -> embassy_net::driver::HardwareAddress {
        self.inner.hardware_address()
    }
}

pub struct FirewallRxToken<R: RxToken, D: Driver> {
    inner: R,
    firewall: *const FirewallDevice<D>,
}

impl<R: RxToken, D: Driver> RxToken for FirewallRxToken<R, D> {
    fn consume<Ret, F>(self, f: F) -> Ret
    where
        F: FnOnce(&mut [u8]) -> Ret,
    {
        let firewall_ptr = self.firewall;
        self.inner.consume(|buffer| {
            if unsafe { (*firewall_ptr).check_packet(buffer) } {
                f(buffer)
            } else {
                // 丢弃包，返回默认值
                f(&mut [])
            }
        })
    }
}