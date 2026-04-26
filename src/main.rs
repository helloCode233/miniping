use clap::Parser;
use socket2::{Domain, Protocol, Socket, Type, SockAddr};
use std::io;
use std::mem::MaybeUninit;
use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, Instant};

const ICMP_ECHO_REQUEST: u8 = 8;
const ICMP_ECHO_REPLY: u8 = 0;

#[derive(Debug)]
struct IcmpHeader {
    type_: u8,
    code: u8,
    checksum: u16,
    identifier: u16,
    sequence: u16,
}

impl IcmpHeader {
    fn new(identifier: u16, sequence: u16) -> Self {
        IcmpHeader {
            type_: ICMP_ECHO_REQUEST,
            code: 0,
            checksum: 0,
            identifier,
            sequence,
        }
    }

    fn serialize(&self) -> [u8; 8] {
        let mut buf = [0u8; 8];
        buf[0] = self.type_;
        buf[1] = self.code;
        buf[2..4].copy_from_slice(&self.checksum.to_be_bytes());
        buf[4..6].copy_from_slice(&self.identifier.to_be_bytes());
        buf[6..8].copy_from_slice(&self.sequence.to_be_bytes());
        buf
    }

    fn deserialize(buf: &[u8]) -> Option<Self> {
        if buf.len() < 8 {
            return None;
        }
        let type_ = buf[0];
        let code = buf[1];
        let checksum = u16::from_be_bytes([buf[2], buf[3]]);
        let identifier = u16::from_be_bytes([buf[4], buf[5]]);
        let sequence = u16::from_be_bytes([buf[6], buf[7]]);
        Some(IcmpHeader {
            type_,
            code,
            checksum,
            identifier,
            sequence,
        })
    }
}

fn checksum(data: &[u8]) -> u16 {
    let mut sum = 0u32;
    let mut i = 0;
    while i + 1 < data.len() {
        sum += u32::from(u16::from_be_bytes([data[i], data[i + 1]]));
        i += 2;
    }
    if i < data.len() {
        sum += u32::from(data[i]) << 8;
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    !(sum as u16)
}

fn build_icmp_echo(identifier: u16, sequence: u16, payload: &[u8]) -> Vec<u8> {
    let header = IcmpHeader::new(identifier, sequence);
    let mut packet = header.serialize().to_vec();
    packet.extend_from_slice(payload);
    let checksum = checksum(&packet);
    packet[2..4].copy_from_slice(&checksum.to_be_bytes());
    packet
}

fn ip_header_len(buf: &[u8]) -> Option<usize> {
    if buf.len() < 1 {
        return None;
    }
    let first = buf[0];
    if first >> 4 == 4 {
        if buf.len() < 20 {
            return None;
        }
        let ihl = (buf[0] & 0x0f) as usize;
        if ihl < 5 {
            return None;
        }
        Some(ihl * 4)
    } else if first >> 4 == 6 {
        Some(40)
    } else {
        Some(0)
    }
}

fn sockaddr_to_ip(addr: &SockAddr) -> Option<IpAddr> {
    if let Some(v4) = addr.as_socket_ipv4() {
        Some(IpAddr::V4(*v4.ip()))
    } else if let Some(v6) = addr.as_socket_ipv6() {
        Some(IpAddr::V6(*v6.ip()))
    } else {
        None
    }
}

// Calculate the ICMP payload size from the total received length
// If IP header is present, subtract it; if not, subtract the 8-byte ICMP header
fn icmp_data_len(total_len: usize, ip_hdr_len: usize) -> usize {
    total_len.saturating_sub(ip_hdr_len).saturating_sub(8)
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target host to ping
    host: String,

    /// Number of echo requests to send
    #[arg(short, long, default_value_t = 4)]
    count: u16,

    /// Timeout in seconds
    #[arg(short, long, default_value_t = 1)]
    timeout: u64,

    /// Interval between requests in milliseconds
    #[arg(short, long, default_value_t = 1000)]
    interval: u64,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    // Resolve host to IP address
    let addr: IpAddr = match args.host.parse() {
        Ok(ip) => ip,
        Err(_) => {
            let addrs = match dns_lookup::lookup_host(&args.host) {
                Ok(addrs) => addrs,
                Err(_) => {
                    eprintln!("miniping: {}: Name or service not known", args.host);
                    std::process::exit(2);
                }
            };
            addrs.into_iter().next().unwrap()
        }
    };

    let host_display = args.host.clone();
    println!("PING {} ({}) 56(84) bytes of data.", host_display, addr);

    let domain = match addr {
        IpAddr::V4(_) => Domain::IPV4,
        IpAddr::V6(_) => Domain::IPV6,
    };
    let protocol = match addr {
        IpAddr::V4(_) => Protocol::ICMPV4,
        IpAddr::V6(_) => Protocol::ICMPV6,
    };

    let socket = Socket::new(domain, Type::DGRAM, Some(protocol))?;
    socket.set_read_timeout(Some(Duration::from_secs(args.timeout)))?;
    socket.set_write_timeout(Some(Duration::from_secs(args.timeout)))?;

    let identifier = std::process::id() as u16;
    let payload = b"abcdefghijklmnopqrstuvwxyz012345"; // 32 bytes payload

    let mut sent = 0;
    let mut received = 0;
    let mut rtts: Vec<f64> = Vec::new();
    let overall_start = Instant::now();

    for seq in 0..args.count {
        let packet = build_icmp_echo(identifier, seq, payload);
        let start = Instant::now();
        let sockaddr = SockAddr::from(SocketAddr::new(addr, 0));
        match socket.send_to(&packet, &sockaddr) {
            Ok(_) => {
                sent += 1;
            }
            Err(e) => {
                eprintln!("miniping: sendto: {}", e);
                continue;
            }
        }

        // Loop receiving until we get a matching reply or timeout
        let mut matched = false;
        while !matched {
            let mut buf = [MaybeUninit::uninit(); 1024];
            match socket.recv_from(&mut buf) {
                Ok((len, src)) => {
                    let elapsed = start.elapsed();
                    let buf_slice =
                        unsafe { std::slice::from_raw_parts(buf.as_ptr() as *const u8, len) };
                    let ip_len = match ip_header_len(buf_slice) {
                        Some(l) => l,
                        None => continue,
                    };
                    if let Some(header) = IcmpHeader::deserialize(&buf_slice[ip_len..]) {
                        if header.type_ == ICMP_ECHO_REPLY && header.sequence == seq {
                            received += 1;
                            matched = true;
                            let rtt = elapsed.as_secs_f64() * 1000.0;
                            rtts.push(rtt);
                            let data_len = icmp_data_len(len, ip_len);
                            let from_ip = sockaddr_to_ip(&src)
                                .map(|ip| ip.to_string())
                                .unwrap_or_else(|| "?".to_string());
                            println!(
                                "{} bytes from {}: icmp_seq={} time={:.1} ms",
                                data_len, from_ip, seq, rtt
                            );
                        }
                    }
                }
                Err(_) => {
                    // Timeout — nothing arrives
                    break;
                }
            }
        }
        if seq + 1 < args.count {
            std::thread::sleep(Duration::from_millis(args.interval));
        }
    }

    let overall_elapsed = overall_start.elapsed();
    let loss_pct = if sent > 0 {
        ((sent - received) as f64 / sent as f64) * 100.0
    } else {
        0.0
    };

    println!();
    println!("--- {} ping statistics ---", host_display);
    println!(
        "{} packets transmitted, {} received, {:.0}% packet loss, time {:.0}ms",
        sent,
        received,
        loss_pct,
        overall_elapsed.as_secs_f64() * 1000.0
    );

    if !rtts.is_empty() {
        let min = rtts.iter().cloned().fold(f64::MAX, f64::min);
        let max = rtts.iter().cloned().fold(f64::MIN, f64::max);
        let avg = rtts.iter().sum::<f64>() / rtts.len() as f64;
        let variance = rtts.iter().map(|v| (v - avg).powi(2)).sum::<f64>() / rtts.len() as f64;
        let mdev = variance.sqrt();
        println!(
            "rtt min/avg/max/mdev = {:.3}/{:.3}/{:.3}/{:.3} ms",
            min, avg, max, mdev
        );
    }

    Ok(())
}