use crate::internal::Memory::*;
use crate::Bytes::*;
use crate::Common::*;
use crate::Math::Vec3;
use libc;

extern "C" {
    fn inet_ntoa(_: libc::in_addr) -> *mut libc::c_char;
    fn inet_aton(_: *const libc::c_char, _: *mut libc::in_addr) -> i32;
}

type sock_t = libc::c_int;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Socket {
    pub type_0: SocketType,
    pub sock: sock_t,
    pub addrSend: libc::sockaddr_in,
    pub addrRecv: libc::sockaddr_in,
    pub buffer: [libc::c_char; 2048],
}

pub type SocketType = i32;

#[inline]
unsafe extern "C" fn _OSSwapInt32(mut _data: u32) -> u32 {
    _data = _data.swap_bytes();
    _data
}

#[inline]
unsafe extern "C" fn _OSSwapInt16(mut _data: u16) -> u16 {
    ((_data as i32) << 8 | _data as i32 >> 8) as u16
}

#[no_mangle]
pub static SocketType_None: SocketType = 0;

#[no_mangle]
pub static SocketType_UDP: SocketType = 0x1;

#[no_mangle]
pub static SocketType_TCP: SocketType = 0x2;

#[inline]
unsafe extern "C" fn Socket_Cleanup(mut this: sock_t) {
    if this != -1 {
        libc::close(this);
    }
}

#[inline]
unsafe extern "C" fn Socket_Receive(
    mut this: sock_t,
    mut buf: *mut libc::c_void,
    mut len: i32,
) -> i32 {
    libc::read(this, buf, len as usize) as i32
}

#[inline]
unsafe extern "C" fn Socket_Send(
    mut this: sock_t,
    mut buf: *const libc::c_void,
    mut len: i32,
) -> i32 {
    libc::write(this, buf, len as usize) as i32
}

#[inline]
unsafe extern "C" fn Socket_SetNonblocking(mut this: sock_t) -> bool {
    libc::fcntl(this, 4, libc::fcntl(this, 3, 0) | 0x4) >= 0
}

#[no_mangle]
pub unsafe extern "C" fn Socket_Create(mut type_0: SocketType) -> *mut Socket {
    if type_0 != SocketType_UDP && type_0 != SocketType_TCP {
        Fatal(
            b"Socket_Create: socket type must be either SocketType_TCP or SocketType_UDP\0"
                as *const u8 as *const libc::c_char,
        );
    }
    let mut this = MemNew!(Socket);
    (*this).type_0 = type_0;
    (*this).sock = libc::socket(
        2,
        if type_0 == SocketType_UDP {
            2
        } else {
            1
        },
        0,
    );
    if (*this).sock == -1 {
        Fatal(b"Socket_Create: failed to open socket\0" as *const u8 as *const libc::c_char);
    }
    let mut opt: i32 = 1;
    if libc::setsockopt(
        (*this).sock,
        0xffff,
        0x4,
        &mut opt as *mut i32 as *mut libc::c_char as *const libc::c_void,
        std::mem::size_of::<i32>() as libc::c_ulong as libc::socklen_t,
    ) != 0
    {
        Fatal(
            b"Socket_Create: failed to set socket to reusable\0" as *const u8
                as *const libc::c_char,
        );
    }
    if !Socket_SetNonblocking((*this).sock) {
        Fatal(
            b"Socket_Create: failed to set socket to non-blocking\0" as *const u8
                as *const libc::c_char,
        );
    }
    this
}

#[no_mangle]
pub unsafe extern "C" fn Socket_Free(mut this: *mut Socket) {
    Socket_Cleanup((*this).sock);
    MemFree(this as *const libc::c_void);
}

#[no_mangle]
pub unsafe extern "C" fn Socket_Accept(mut this: *mut Socket) -> *mut Socket {
    if (*this).type_0 != SocketType_TCP {
        Fatal(
            b"Socket_Accept: can only accept connections on TCP sockets\0" as *const u8
                as *const libc::c_char,
        );
    }
    let mut sock: sock_t = libc::accept((*this).sock, std::ptr::null_mut(), std::ptr::null_mut());
    if sock == -1 {
        return std::ptr::null_mut();
    }
    let mut con = MemNew!(Socket);
    (*con).type_0 = SocketType_TCP;
    (*con).sock = sock;
    if !Socket_SetNonblocking((*con).sock) {
        Fatal(
            b"Socket_Accept: failed to set socket to non-blocking\0" as *const u8
                as *const libc::c_char,
        );
    }
    con
}

#[no_mangle]
pub unsafe extern "C" fn Socket_Bind(mut this: *mut Socket, mut port: i32) {
    let mut addr: libc::sockaddr_in = libc::sockaddr_in {
        sin_len: 0,
        sin_family: 0,
        sin_port: 0,
        sin_addr: libc::in_addr { s_addr: 0 },
        sin_zero: [0; 8],
    };
    MemSet(
        &mut addr as *mut libc::sockaddr_in as *mut libc::c_void,
        0,
        std::mem::size_of::<libc::sockaddr_in>(),
    );
    addr.sin_family = 2 as libc::sa_family_t;
    addr.sin_port = (if 0 != 0 {
        ((port as u16 as u32 & 0xff00) >> 8 | (port as u16 as u32 & 0xff) << 8)
            as u16 as i32
    } else {
        _OSSwapInt16(port as u16) as i32
    }) as u16;
    addr.sin_addr.s_addr = if 0 != 0 {
        (0 & 0xff000000) >> 24
            | (0 & 0xff0000) >> 8
            | (0 & 0xff00) << 8
            | (0 & 0xff) << 24
    } else {
        _OSSwapInt32(0)
    };
    if libc::bind(
        (*this).sock,
        &mut addr as *mut libc::sockaddr_in as *const libc::sockaddr,
        std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t,
    ) == -1
    {
        Fatal(b"Socket_Bind: failed to bind socket\0" as *const u8 as *const libc::c_char);
    }
}

#[no_mangle]
pub unsafe extern "C" fn Socket_Listen(mut this: *mut Socket) {
    if (*this).type_0 != SocketType_TCP {
        Fatal(
            b"Socket_Listen: can only listen for connections on TCP sockets\0" as *const u8
                as *const libc::c_char,
        );
    }
    if libc::listen((*this).sock, 1) == -1 {
        Fatal(b"Socket_Listen: failed to listen\0" as *const u8 as *const libc::c_char);
    }
}

#[no_mangle]
pub unsafe extern "C" fn Socket_Read(mut this: *mut Socket) -> *const libc::c_char {
    let mut bytes: i32 = Socket_Receive(
        (*this).sock,
        ((*this).buffer).as_mut_ptr() as *mut libc::c_void,
        std::mem::size_of::<[libc::c_char; 2048]>() as libc::c_ulong as i32,
    );
    if bytes == -1 {
        if std::io::Error::last_os_error().raw_os_error().unwrap_or(0) == 35 {
            return std::ptr::null();
        }
        Fatal(b"Socket_Read: failed to read from socket\0" as *const u8 as *const libc::c_char);
    }
    if bytes == 0 {
        return std::ptr::null();
    }
    (*this).buffer[bytes as usize] = 0 as libc::c_char;
    ((*this).buffer).as_mut_ptr() as *const libc::c_char
}

#[no_mangle]
pub unsafe extern "C" fn Socket_ReadBytes(mut this: *mut Socket) -> *mut Bytes {
    let mut bytes: i32 = Socket_Receive(
        (*this).sock,
        ((*this).buffer).as_mut_ptr() as *mut libc::c_void,
        std::mem::size_of::<[libc::c_char; 2048]>() as libc::c_ulong as i32,
    );
    if bytes == -1 {
        if std::io::Error::last_os_error().raw_os_error().unwrap_or(0) == 35 {
            return std::ptr::null_mut();
        }
        Fatal(b"Socket_ReadRaw: failed to read from socket\0" as *const u8 as *const libc::c_char);
    }
    if bytes == 0 {
        return std::ptr::null_mut();
    }
    let mut data: *mut Bytes = Bytes_Create(bytes as u32);
    Bytes_Write(
        data,
        ((*this).buffer).as_mut_ptr() as *const libc::c_void,
        bytes as u32,
    );
    data
}

#[no_mangle]
pub unsafe extern "C" fn Socket_ReceiveFrom(
    mut this: *mut Socket,
    mut data: *mut libc::c_void,
    mut len: usize,
) -> i32 {
    MemZero(data, len);
    let mut addrSize: libc::socklen_t = 0;
    let mut bytes: i32 = libc::recvfrom(
        (*this).sock,
        data,
        len,
        0,
        &mut (*this).addrRecv as *mut libc::sockaddr_in as *mut libc::sockaddr,
        &mut addrSize,
    ) as i32;
    if bytes == -1 {
        if std::io::Error::last_os_error().raw_os_error().unwrap_or(0) == 35 {
            return 0;
        }
        return -1;
    }
    bytes
}

#[no_mangle]
pub unsafe extern "C" fn Socket_GetAddress(mut this: *mut Socket) -> *const libc::c_char {
    StrFormat(
        b"%s:%d\0" as *const u8 as *const libc::c_char,
        inet_ntoa((*this).addrRecv.sin_addr),
        (if 0 != 0 {
            (((*this).addrRecv.sin_port as u32 & 0xff00) >> 8
                | ((*this).addrRecv.sin_port as u32 & 0xff) << 8) as u16 as i32
        } else {
            _OSSwapInt16((*this).addrRecv.sin_port) as i32
        }) as u16 as i32,
    )
}

#[no_mangle]
pub unsafe extern "C" fn Socket_SetAddress(mut this: *mut Socket, mut addr: *const libc::c_char) {
    let mut colon: *const libc::c_char = StrFind(addr, b":\0" as *const u8 as *const libc::c_char);
    if colon.is_null() {
        Fatal(
            b"Socket_SetReceiver: address must be in format a.b.c.d:port format\0" as *const u8
                as *const libc::c_char,
        );
    }
    let mut ip: *const libc::c_char = StrSubStr(addr, colon);
    let mut port: *const libc::c_char = StrSubStr(colon.offset(1), addr.add(libc::strlen(addr)));
    (*this).addrSend.sin_family = 2 as libc::sa_family_t;
    (*this).addrSend.sin_port = (if 0 != 0 {
        ((libc::strtol(port, std::ptr::null_mut(), 0) as u16 as u32 & 0xff00) >> 8
            | (libc::strtol(port, std::ptr::null_mut(), 0) as u16 as u32 & 0xff) << 8)
            as u16 as i32
    } else {
        _OSSwapInt16(libc::strtol(port, std::ptr::null_mut(), 0) as u16) as i32
    }) as u16;
    if inet_aton(ip, &mut (*this).addrSend.sin_addr) == 0 {
        Fatal(
            b"Socket_SetReceiver: failed to interpret network address\0" as *const u8
                as *const libc::c_char,
        );
    }
    StrFree(ip);
    StrFree(port);
}

#[no_mangle]
pub unsafe extern "C" fn Socket_SendTo(
    mut this: *mut Socket,
    mut data: *const libc::c_void,
    mut len: usize,
) -> i32 {
    let mut bytes: i32 = libc::sendto(
        (*this).sock,
        data,
        len,
        0,
        &mut (*this).addrSend as *mut libc::sockaddr_in as *const libc::sockaddr,
        std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t,
    ) as i32;
    if bytes == -1 {
        return -1;
    }
    bytes
}

#[no_mangle]
pub unsafe extern "C" fn Socket_Write(mut this: *mut Socket, mut msg: *const libc::c_char) {
    if Socket_Send((*this).sock, msg as *const libc::c_void, msg as i32) == -1 {
        Fatal(b"Socket_Write: failed to write to socket\0" as *const u8 as *const libc::c_char);
    }
}

#[no_mangle]
pub unsafe extern "C" fn Socket_WriteBytes(mut this: *mut Socket, mut msg: *mut Bytes) {
    if Socket_Send((*this).sock, Bytes_GetData(msg), Bytes_GetSize(msg) as i32) == -1 {
        Fatal(b"Socket_WriteRaw: failed to write to socket\0" as *const u8 as *const libc::c_char);
    }
}
