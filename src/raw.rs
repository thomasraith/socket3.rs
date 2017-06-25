extern crate libc;

use std::io;
use std::net;
use std::mem;

#[allow(non_snake_case)]
pub mod Domain {
    use super::libc;
    pub const IPV4 : libc::c_int = libc::AF_INET;
    pub const IPV6 : libc::c_int = libc::AF_INET6;
    pub const PACKET : libc::c_int = libc::AF_PACKET;
}

#[allow(non_snake_case)]
pub mod Protocol {
    use super::libc;
    pub const RAW : libc::c_int = libc::IPPROTO_RAW;
    #[cfg(target_endian = "little")]
    pub const ETH_ALL : u16 = 0x0300;                           //TODO: use htons
    #[cfg(target_endian = "big")]
    pub const ETH_ALL : u16 = 0x0003;
}

type SOCKET = libc::c_int;

pub struct Socket {
    fd: SOCKET
}

impl Socket {
    pub fn new(domain : libc::c_int, protocol : libc::c_int) -> io::Result<Socket> {
        unsafe {
            match libc::socket(domain, libc::SOCK_RAW, protocol) {
                -1 => Err(io::Error::last_os_error()),
                fd => Ok(Socket {
                    fd: fd
                })
            }
        }
    }

    pub fn send_to(&self, buffer : &[u8], dest_addr : net::SocketAddr, flags : libc::c_int) -> io::Result<usize> {
        let length = buffer.len();
        let (dst_addr, dst_addr_len) = to_c_sockaddr_struct(dest_addr);

        unsafe {
            match libc::sendto(self.fd, buffer.as_ptr() as *const libc::c_void, length, flags, dst_addr, dst_addr_len) {
                -1 => {
                    let system_error = io::Error::last_os_error();

                    match system_error.raw_os_error().unwrap() {
                        libc::ESHUTDOWN => Ok(0),
                        _ => Err(system_error)
                    }
                },
                n => Ok(n as usize)
            }
        }
    }

    pub fn send(&self, buffer : &[u8], flags : libc::c_int) -> io::Result<usize> {
        unsafe {
            match libc::send(self.fd, buffer.as_ptr() as *const libc::c_void, buffer.len(), flags) {
                -1 => {
                    let system_error = io::Error::last_os_error();

                    match system_error.raw_os_error().unwrap() {
                        libc::ESHUTDOWN => Ok(0),
                        _ => Err(system_error)
                    }
                },
                n => Ok(n as usize)
            }
        }
    }

    pub fn receive(&self, buffer : &mut [u8], flags : libc::c_int) -> io::Result<usize> {
        unsafe {
            match libc::recv(self.fd, buffer.as_mut_ptr() as *mut libc::c_void, buffer.len(), flags) {
                -1 => Err(io::Error::last_os_error()),
                n => Ok(n as usize)
            }
        }
    }

    pub fn receive_from(&self, buffer : &mut [u8], flags : libc::c_int) -> io::Result<(usize, net::SocketAddr)> {
        unsafe {
            let mut dst_addr : libc::sockaddr_storage = mem::zeroed();
            let mut dst_addr_len : libc::socklen_t = mem::size_of_val(&dst_addr) as libc::socklen_t;

            match libc::recvfrom(self.fd, buffer.as_mut_ptr() as *mut libc::c_void, buffer.len(), flags, &mut dst_addr as *mut _ as *mut _, &mut dst_addr_len) {
                -1 => Err(io::Error::last_os_error()),
                n => Ok((n as usize, to_rust_socketaddr(&dst_addr)?))
            }
        }
    }

    pub fn close(&self) -> io::Result<()> {
        unsafe {
            match libc::close(self.fd) {
                0 => Ok(()),
                _ => Err(io::Error::last_os_error())
            }
        }
    }
}

fn to_c_sockaddr_struct(address : net::SocketAddr) -> (*const libc::sockaddr, libc::socklen_t) {
    match address {
        net::SocketAddr::V4(ref addr) => 
            (addr as *const _ as *const _, mem::size_of_val(addr) as libc::socklen_t),
        net::SocketAddr::V6(ref addr) => 
            (addr as *const _ as *const _, mem::size_of_val(addr) as libc::socklen_t)
        
    }
}

fn to_rust_socketaddr(c_struct : &libc::sockaddr_storage) -> io::Result<net::SocketAddr> {
    match c_struct.ss_family as libc::c_int {
        libc::AF_INET => {
            let ipv4_c_struct = unsafe { *(c_struct as *const _ as *const libc::sockaddr_in) };
            let ipv4_addr = unsafe { *(&ipv4_c_struct as *const _ as *const _ as *const [u8; 4]) };
            let ipv4_addr = net::Ipv4Addr::from(ipv4_addr);

            #[cfg(target_endian = "little")]
            let socket_addr = net::SocketAddrV4::new(ipv4_addr, ipv4_c_struct.sin_port.to_be());
            #[cfg(target_endian = "big")]
            let socket_addr = net::SocketAddrV4::new(ipv4_addr, ipv4_c_struct.sin_port);

            Ok(net::SocketAddr::V4(socket_addr))
        },
        libc::AF_INET6 => {
            let ipv6_c_struct = unsafe { *(c_struct as *const _ as *const libc::sockaddr_in6) };
            let ipv6_addr = net::Ipv6Addr::from(ipv6_c_struct.sin6_addr.s6_addr.clone());

            #[cfg(target_endian = "little")]
            let socket_addr = net::SocketAddrV6::new(ipv6_addr, ipv6_c_struct.sin6_port.to_be(), ipv6_c_struct.sin6_flowinfo, ipv6_c_struct.sin6_scope_id);
            #[cfg(target_endian = "big")]
            let socket_addr = net::SocketAddrV6::new(ipv6_addr, ipv6_c_struct.sin6_port, ipv6_c_struct.sin6_flowinfo, ipv6_c_struct.sin6_scope_id);

            Ok(net::SocketAddr::V6(socket_addr))
        },
        _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "Unknown address type! This version only supports IPv4 and IPv6."))
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
