//
// Socket stuff
//
use crate::{AF_UNSPEC, SOCK_DGRAM, SOCK_STREAM};
use alloc::boxed::Box;
use alloc::string::ToString;
use axerrno::{AxError, AxResult};
use ruxnet::SocketAddr;
use ruxnet::TcpSocket;
use ruxnet::UdpSocket;
use core::net::{IpAddr, SocketAddr as StdSocketAddr};
use core::str::FromStr;

/// A handle to a TCP socket.
pub struct AxTcpSocketHandle(TcpSocket);

/// A handle to a UDP socket.
pub struct AxUdpSocketHandle(UdpSocket);

////////////////////////////////////////////////////////////////////////////////
// TCP socket
////////////////////////////////////////////////////////////////////////////////

pub fn tcp_socket() -> AxTcpSocketHandle {
    AxTcpSocketHandle(TcpSocket::new())
}

pub fn tcp_socket_addr(socket: &AxTcpSocketHandle) -> AxResult<StdSocketAddr> {
    socket.0.local_addr()
}

pub fn tcp_peer_addr(socket: &AxTcpSocketHandle) -> AxResult<StdSocketAddr> {
    socket.0.peer_addr()
}

pub fn tcp_connect(socket: &AxTcpSocketHandle, addr: StdSocketAddr) -> AxResult {
    socket.0.connect(addr)
}

pub fn tcp_bind(socket: &AxTcpSocketHandle, addr: StdSocketAddr) -> AxResult {
    socket.0.bind(addr)
}

pub fn tcp_listen(socket: &AxTcpSocketHandle, _backlog: usize) -> AxResult {
    socket.0.listen()
}

pub fn tcp_accept(socket: &AxTcpSocketHandle) -> AxResult<(AxTcpSocketHandle, StdSocketAddr)> {
    let new_sock = socket.0.accept()?;
    let addr = new_sock.peer_addr()?;
    Ok((AxTcpSocketHandle(new_sock), addr))
}

pub fn tcp_send(socket: &AxTcpSocketHandle, buf: &[u8]) -> AxResult<usize> {
    socket.0.send(buf)
}

pub fn tcp_recv(socket: &AxTcpSocketHandle, buf: &mut [u8]) -> AxResult<usize> {
    socket.0.recv(buf, 0)
}

pub fn tcp_shutdown(socket: &AxTcpSocketHandle) -> AxResult {
    socket.0.shutdown()
}

pub fn get_addr_info(
    domain_name: &str,
    port: Option<u16>,
) -> AxResult<alloc::vec::Vec<StdSocketAddr>> {
    let domain_to_query = if domain_name == "localhost" {
        "127.0.0.1"
    } else {
        domain_name
    };

    Ok(ruxnet::dns_query(domain_to_query)?
        .into_iter()
        .map(|ip| StdSocketAddr::new(ip, port.unwrap_or(0)))
        .collect())
}

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
            let ret = sock.recv(buf, 0).unwrap();
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
    let ips = ruxnet::dns_query(name).unwrap();
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

const fn into_core_ipaddr(ip: ruxnet::IpAddr) -> IpAddr {
    match ip {
        ruxnet::IpAddr::Ipv4(ip) => IpAddr::V4(unsafe { core::mem::transmute(ip.0) }),
    }
}

const fn into_ax_ipaddr(ip: IpAddr) -> ruxnet::IpAddr {
    match ip {
        IpAddr::V4(ip) => ruxnet::IpAddr::Ipv4(ruxnet::Ipv4Addr(ip.octets())),
        _ => panic!("IPv6 not supported"),
    }
}


fn sockaddr_std_to_ax(addr: &StdSocketAddr) -> SocketAddr {
    let s = addr.ip().to_string();
    let s = ruxnet::IpAddr::from_str(&s).unwrap();
    SocketAddr::new(s, addr.port())
}

fn sockaddr_ax_to_std(addr: &SocketAddr) -> StdSocketAddr {
    let s = addr.addr.to_string();
    let s = IpAddr::from_str(&s).unwrap();
    StdSocketAddr::new(s, addr.port)
}