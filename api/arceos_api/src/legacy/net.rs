//
// Socket stuff
//
use crate::legacy::{AF_UNSPEC, SOCK_DGRAM, SOCK_STREAM};
use alloc::boxed::Box;
use alloc::string::ToString;
use axerrno::AxError;
use axnet::SocketAddr;
use axnet::TcpSocket;
use axnet::UdpSocket;
use core::net::{IpAddr, SocketAddr as StdSocketAddr};
use core::str::FromStr;

enum StdSocketWrap {
    Tcp(TcpSocket),
    Udp(UdpSocket),
}

#[no_mangle]
pub fn sys_socket(family: i32, ty: i32) -> usize {
    assert!(family == AF_UNSPEC, "bad family {}", family);

    let sock = match ty {
        SOCK_STREAM => {
            axlog::debug!("sys_socket: tcp");
            StdSocketWrap::Tcp(TcpSocket::new())
        }
        SOCK_DGRAM => {
            axlog::debug!("sys_socket: udp");
            StdSocketWrap::Udp(UdpSocket::new())
        }
        _ => {
            panic!("bad socket type '{}'.", ty);
        }
    };
    let ptr = Box::leak(Box::new(sock));
    ptr as *mut _ as usize
}

#[no_mangle]
pub fn sys_bind(s: usize, addr: &StdSocketAddr) {
    // let addr = sockaddr_std_to_ax(addr);

    let f = s as *mut StdSocketWrap;
    let wrap = unsafe { f.as_mut().unwrap() };
    match wrap {
        StdSocketWrap::Tcp(sock) => {
            axlog::debug!("sys_bind: tcp {:?}", addr);
            let _ = sock.bind(*addr);
        }
        StdSocketWrap::Udp(sock) => {
            let _ = sock.bind(*addr);
        }
    }
}

/// listen for connections on a socket
///
/// The `backlog` parameter defines the maximum length for the queue of pending
/// connections. Currently, the `backlog` must be one.
#[no_mangle]
pub fn sys_listen(s: usize, _backlog: i32) -> i32 {
    let f = s as *mut StdSocketWrap;
    let wrap = unsafe { f.as_mut().unwrap() };
    match wrap {
        StdSocketWrap::Tcp(sock) => {
            axlog::debug!("sys_listen: ");
            let _ = sock.listen();
            0
        }
        StdSocketWrap::Udp(_) => {
            panic!("sys_listen: udp");
        }
    }
}

#[no_mangle]
pub fn sys_getsockname(s: usize) -> Result<StdSocketAddr, AxError> {
    let f = s as *mut StdSocketWrap;
    let wrap = unsafe { f.as_mut().unwrap() };
    match wrap {
        StdSocketWrap::Tcp(sock) => {
            let ret = sock.local_addr()?;
            Ok(ret)
        }
        StdSocketWrap::Udp(sock) => {
            let ret = sock.local_addr()?;
            Ok(ret)
        }
    }
}

#[no_mangle]
pub fn sys_accept(s: usize) -> Result<(usize, StdSocketAddr), AxError> {
    let f = s as *mut StdSocketWrap;
    let wrap = unsafe { f.as_mut().unwrap() };
    match wrap {
        StdSocketWrap::Tcp(sock) => {
            axlog::debug!("sys_accept: ");
            let sock = sock.accept()?;
            let addr = sock.peer_addr()?;
            // let addr = sockaddr_ax_to_std(&addr);
            let sock = StdSocketWrap::Tcp(sock);
            axlog::debug!("sys_accept: {:?}", addr);
            let ptr = Box::leak(Box::new(sock));
            Ok((ptr as *mut _ as usize, addr))
        }
        StdSocketWrap::Udp(_) => {
            panic!("sys_accept: udp");
        }
    }
}

#[no_mangle]
pub fn sys_recv(s: usize, buf: &mut [u8], _flags: i32) -> usize {
    axlog::debug!("sys_recv: ...");
    let f = s as *mut StdSocketWrap;
    let wrap = unsafe { f.as_mut().unwrap() };
    match wrap {
        StdSocketWrap::Tcp(sock) => {
            axlog::debug!("sys_recv: tcp");
            let ret = sock.recv(buf).unwrap();
            axlog::debug!("sys_recv: ret{}", ret);
            ret
        }
        StdSocketWrap::Udp(_) => {
            panic!("sys_read: ");
        }
    }
}

#[no_mangle]
pub fn sys_send(s: usize, buf: &[u8]) -> usize {
    axlog::debug!("sys_send: ...");
    let f = s as *mut StdSocketWrap;
    let wrap = unsafe { f.as_mut().unwrap() };
    match wrap {
        StdSocketWrap::Tcp(sock) => {
            axlog::debug!("sys_send: ...");
            let ret = sock.send(buf).unwrap();
            axlog::debug!("sys_send: ok! ret {}", ret);
            ret
        }
        StdSocketWrap::Udp(_) => {
            panic!("sys_send: ");
        }
    }
}

#[no_mangle]
pub fn sys_connect(s: usize, addr: &StdSocketAddr) {
    // let addr = sockaddr_std_to_ax(addr);

    let f = s as *mut StdSocketWrap;
    let wrap = unsafe { f.as_mut().unwrap() };
    match wrap {
        StdSocketWrap::Tcp(sock) => {
            axlog::debug!("sys_connect {:?}", addr);
            sock.connect(*addr).unwrap()
        }
        StdSocketWrap::Udp(_) => {
            panic!("sys_connect: ");
        }
    }
}

#[no_mangle]
pub fn sys_recvfrom(s: usize, buf: &mut [u8], _flags: i32) -> (usize, StdSocketAddr) {
    let f = s as *mut StdSocketWrap;
    let wrap = unsafe { f.as_mut().unwrap() };
    let (num, addr) = match wrap {
        StdSocketWrap::Tcp(_) => {
            panic!("sys_recvfrom: ");
        }
        StdSocketWrap::Udp(sock) => sock.recv_from(buf).unwrap(),
    };
    // let addr = sockaddr_ax_to_std(&addr);
    (num, addr)
}

#[no_mangle]
pub fn sys_sendto(s: usize, buf: &[u8], dst: &StdSocketAddr) -> usize {
    // let dst = sockaddr_std_to_ax(dst);

    let f = s as *mut StdSocketWrap;
    let wrap = unsafe { f.as_mut().unwrap() };
    match wrap {
        StdSocketWrap::Tcp(_) => {
            panic!("sys_sendto: ");
        }
        StdSocketWrap::Udp(sock) => sock.send_to(buf, *dst).unwrap(),
    }
}

#[no_mangle]
pub fn sys_getaddrinfo(name: &str, port: u16) -> Result<alloc::vec::Vec<StdSocketAddr>, AxError> {
    let mut ret: alloc::vec::Vec<StdSocketAddr> = alloc::vec![];
    let ips = axnet::dns_query(name).unwrap();
    for ip in ips {
        let s: SocketAddr = SocketAddr::new(into_ax_ipaddr(ip), port);
        let s = sockaddr_ax_to_std(&s);
        ret.push(s);
    }
    Ok(ret)
}

#[no_mangle]
pub fn sys_close_socket(handle: usize) {
    unsafe { core::ptr::drop_in_place(handle as *mut StdSocketWrap) }
}

const fn into_core_ipaddr(ip: axnet::IpAddr) -> IpAddr {
    match ip {
        axnet::IpAddr::Ipv4(ip) => IpAddr::V4(unsafe { core::mem::transmute(ip.0) }),
    }
}

const fn into_ax_ipaddr(ip: IpAddr) -> axnet::IpAddr {
    match ip {
        IpAddr::V4(ip) => axnet::IpAddr::Ipv4(axnet::Ipv4Addr(ip.octets())),
        _ => panic!("IPv6 not supported"),
    }
}


fn sockaddr_std_to_ax(addr: &StdSocketAddr) -> SocketAddr {
    let s = addr.ip().to_string();
    let s = axnet::IpAddr::from_str(&s).unwrap();
    SocketAddr::new(s, addr.port())
}

fn sockaddr_ax_to_std(addr: &SocketAddr) -> StdSocketAddr {
    let s = addr.addr.to_string();
    let s = IpAddr::from_str(&s).unwrap();
    StdSocketAddr::new(s, addr.port)
}