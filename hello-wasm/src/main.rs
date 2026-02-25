use std::io::{self, Read, Write};
use std::os::wasi::io::FromRawFd;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct WasiAddrIp4 {
    pub n0: u8,
    pub n1: u8,
    pub n2: u8,
    pub n3: u8,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct WasiAddrIp4Port {
    pub addr: WasiAddrIp4,
    pub port: u16,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct WasiAddrIp6Port {
    pub pad: [u8; 18], // Just padding to match C union size in WAMR
}

#[repr(C)]
pub union WasiAddrUnion {
    pub ip4: WasiAddrIp4Port,
    pub ip6: WasiAddrIp6Port,
}

#[repr(C)]
pub struct WasiAddr {
    pub kind: u32, // 0 for IPv4, 1 for IPv6
    pub addr: WasiAddrUnion,
}

#[link(wasm_import_module = "wasi_snapshot_preview1")]
extern "C" {
    // WAMR's signature is (poolfd, af, socktype, *sockfd) -> errno
    fn sock_open(poolfd: i32, af: i32, socktype: i32, sockfd: *mut i32) -> i32;
    // WAMR's signature is (sockfd, *addr) -> errno
    fn sock_connect(sockfd: i32, addr: *const WasiAddr) -> i32;
    // WAMR's signature is (sockfd, *si_data, si_data_len, si_flags, *so_data_len) -> errno
    fn sock_send(sockfd: i32, si_data: *const WasiCiovec, si_data_len: i32, si_flags: i32, so_data_len: *mut i32) -> i32;
    // WAMR's signature is (sockfd, *ri_data, ri_data_len, ri_flags, *ro_data_len, *ro_flags) -> errno
    fn sock_recv(sockfd: i32, ri_data: *const WasiIovec, ri_data_len: i32, ri_flags: i32, ro_data_len: *mut i32, ro_flags: *mut i32) -> i32;
}

#[repr(C)]
pub struct WasiCiovec {
    pub buf: *const u8,
    pub buf_len: usize,
}

#[repr(C)]
pub struct WasiIovec {
    pub buf: *mut u8,
    pub buf_len: usize,
}

pub struct WamrTcpStream {
    fd: i32,
}

impl WamrTcpStream {
    pub fn connect(ip: [u8; 4], port: u16) -> Result<Self, i32> {
        let mut fd: i32 = -1;
        // AF_INET = 0, SOCK_STREAM = 1, poolfd = -1 (ignored by WAMR)
        let res = unsafe { sock_open(-1, 0, 1, &mut fd as *mut i32) };
        if res != 0 {
            return Err(res);
        }

        let addr = WasiAddr {
            kind: 0, // IPv4
            addr: WasiAddrUnion {
                ip4: WasiAddrIp4Port {
                    // WAMR expects host byte order port initially, or it maps it? 
                    // Let's pass it as is, or we might need to convert it depending on WAMR's expectation.
                    // Typical WASI converts to network host on the polyfill side, let's just pass `port` and if it fails, try swapping.
                    addr: WasiAddrIp4 {
                        n0: ip[0],
                        n1: ip[1],
                        n2: ip[2],
                        n3: ip[3],
                    },
                    port,
                },
            },
        };

        let res = unsafe { sock_connect(fd, &addr as *const WasiAddr) };
        if res != 0 {
            return Err(res);
        }

        Ok(WamrTcpStream { fd })
    }
}

impl Write for WamrTcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let vec = WasiCiovec {
            buf: buf.as_ptr(),
            buf_len: buf.len(),
        };
        let mut written: i32 = 0;
        let res = unsafe { sock_send(self.fd, &vec as *const WasiCiovec, 1, 0, &mut written as *mut i32) };
        if res != 0 {
            Err(io::Error::from_raw_os_error(res))
        } else {
            Ok(written as usize)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Read for WamrTcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let vec = WasiIovec {
            buf: buf.as_mut_ptr(),
            buf_len: buf.len(),
        };
        let mut read_bytes: i32 = 0;
        let mut ro_flags: i32 = 0;
        let res = unsafe { sock_recv(self.fd, &vec as *const WasiIovec, 1, 0, &mut read_bytes as *mut i32, &mut ro_flags as *mut i32) };
        if res != 0 {
            Err(io::Error::from_raw_os_error(res))
        } else {
            Ok(read_bytes as usize)
        }
    }
}

fn main() {
    println!("Hello from WASM on Zephyr OCRE! Executing custom WAMR bindings.");

    match WamrTcpStream::connect([10, 0, 2, 2], 8080) {
        Ok(mut stream) => {
            println!("WAMR Socket Connected!");
            
            let req = "GET / HTTP/1.1\r\nHost: 10.0.2.2\r\nConnection: close\r\n\r\n";
            if let Err(e) = stream.write_all(req.as_bytes()) {
                println!("Failed to write: {}", e);
                return;
            }
            
            let mut res = String::new();
            match stream.read_to_string(&mut res) {
                Ok(_) => {
                    println!("Cloud Response:\n{}", res);
                    println!("Edge-to-Cloud Success!");
                }
                Err(e) => println!("Failed to read: {}", e),
            }
        }
        Err(e) => {
            println!("WAMR sock_connect failed with WASI errno: {}", e);
        }
    }
}
