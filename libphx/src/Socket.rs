use crate::internal::Memory::*;
use glam::Vec3;
use libc;
extern "C" {
    pub type Bytes;
    fn Bytes_GetSize(_: *mut Bytes) -> u32;
    fn strlen(_: *const libc::c_char) -> libc::c_ulong;
    fn strstr(_: *const libc::c_char, _: *const libc::c_char) -> *mut libc::c_char;
    fn Bytes_GetData(_: *mut Bytes) -> *mut libc::c_void;
    fn Fatal(_: *const libc::c_char, _: ...);
    fn Bytes_Create(len: u32) -> *mut Bytes;
    fn Bytes_Write(_: *mut Bytes, data: *const libc::c_void, len: u32);
    fn strtol(_: *const libc::c_char, _: *mut *mut libc::c_char, _: i32) -> libc::c_long;
    fn fcntl(_: i32, _: i32, _: ...) -> i32;
    fn close(_: i32) -> i32;
    fn read(_: i32, _: *mut libc::c_void, _: usize) -> isize;
    fn write(__fd: i32, __buf: *const libc::c_void, __nbyte: usize) -> isize;
    fn accept(_: i32, _: *mut sockaddr, _: *mut socklen_t) -> i32;
    fn bind(_: i32, _: *const sockaddr, _: socklen_t) -> i32;
    fn listen(_: i32, _: i32) -> i32;
    fn recvfrom(
        _: i32,
        _: *mut libc::c_void,
        _: usize,
        _: i32,
        _: *mut sockaddr,
        _: *mut socklen_t,
    ) -> isize;
    fn sendto(
        _: i32,
        _: *const libc::c_void,
        _: usize,
        _: i32,
        _: *const sockaddr,
        _: socklen_t,
    ) -> isize;
    fn setsockopt(_: i32, _: i32, _: i32, _: *const libc::c_void, _: socklen_t) -> i32;
    fn socket(_: i32, _: i32, _: i32) -> i32;
    fn inet_ntoa(_: in_addr) -> *mut libc::c_char;
    fn inet_aton(_: *const libc::c_char, _: *mut in_addr) -> i32;
}
pub type __builtin_va_list = *mut libc::c_char;
pub type __u8_t = libc::c_uchar;
pub type __u16 = u16;
pub type __u32 = u32;
pub type __darwin_socklen_t = __u32;
pub type u_i32_t = u32;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Socket {
    pub type_0: SocketType,
    pub sock: sock_t,
    pub addrSend: sockaddr_in,
    pub addrRecv: sockaddr_in,
    pub buffer: [libc::c_char; 2048],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sockaddr_in {
    pub sin_len: __u8_t,
    pub sin_family: sa_family_t,
    pub sin_port: in_port_t,
    pub sin_addr: in_addr,
    pub sin_zero: [libc::c_char; 8],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct in_addr {
    pub s_addr: in_addr_t,
}
pub type in_addr_t = __u32;
pub type in_port_t = __u16;
pub type sa_family_t = __u8_t;
pub type sock_t = i32;
pub type SocketType = i32;
pub type socklen_t = __darwin_socklen_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sockaddr {
    pub sa_len: __u8_t,
    pub sa_family: sa_family_t,
    pub sa_data: [libc::c_char; 14],
}
pub type va_list = __builtin_va_list;
#[inline]
unsafe extern "C" fn _OSSwapInt32(mut _data: u32) -> u32 {
    _data = _data.swap_bytes();
    return _data;
}
#[inline]
unsafe extern "C" fn _OSSwapInt16(mut _data: u16) -> u16 {
    return ((_data as i32) << 8 as i32 | _data as i32 >> 8 as i32) as u16;
}

#[no_mangle]
pub static mut SocketType_None: SocketType = 0 as i32;
#[no_mangle]
pub static mut SocketType_UDP: SocketType = 0x1 as i32;
#[no_mangle]
pub static mut SocketType_TCP: SocketType = 0x2 as i32;

#[inline]
unsafe extern "C" fn StrFind(mut s: *const libc::c_char, mut sub: *const libc::c_char) -> *const libc::c_char {
    return strstr(s, sub) as *const libc::c_char;
}
#[inline]
unsafe extern "C" fn StrSubStr(mut begin: *const libc::c_char, mut end: *const libc::c_char) -> *const libc::c_char {
    let mut len: usize = end.offset_from(begin) as libc::c_long as usize;
    let mut result: *mut libc::c_char = StrAlloc(len.wrapping_add(1 as usize));
    let mut pResult: *mut libc::c_char = result;
    while begin != end {
        let fresh0 = begin;
        begin = begin.offset(1);
        let fresh1 = pResult;
        pResult = pResult.offset(1);
        *fresh1 = *fresh0;
    }
    *result.offset(len as isize) = 0 as i32 as libc::c_char;
    return result as *const libc::c_char;
}
#[inline]
unsafe extern "C" fn Socket_Cleanup(mut this: sock_t) {
    if this != -(1 as i32) {
        close(this);
    }
}
#[inline]
unsafe extern "C" fn Socket_Receive(
    mut this: sock_t,
    mut buf: *mut libc::c_void,
    mut len: i32,
) -> i32 {
    return read(this, buf, len as usize) as i32;
}
#[inline]
unsafe extern "C" fn Socket_Send(
    mut this: sock_t,
    mut buf: *const libc::c_void,
    mut len: i32,
) -> i32 {
    return write(this, buf, len as usize) as i32;
}
#[inline]
unsafe extern "C" fn Socket_SetNonblocking(mut this: sock_t) -> bool {
    return fcntl(this, 4 as i32, fcntl(this, 3 as i32, 0 as i32) | 0x4 as i32) >= 0 as i32;
}
#[no_mangle]
pub unsafe extern "C" fn Socket_Create(mut type_0: SocketType) -> *mut Socket {
    if type_0 != SocketType_UDP && type_0 != SocketType_TCP {
        Fatal(
            b"Socket_Create: socket type must be either SocketType_TCP or SocketType_UDP\0"
                as *const u8 as *const libc::c_char,
        );
    }
    let mut this: *mut Socket = MemAlloc(::core::mem::size_of::<Socket>() as usize) as *mut Socket;
    (*this).type_0 = type_0;
    (*this).sock = socket(
        2 as i32,
        if type_0 == SocketType_UDP {
            2 as i32
        } else {
            1 as i32
        },
        0 as i32,
    );
    if (*this).sock == -(1 as i32) {
        Fatal(b"Socket_Create: failed to open socket\0" as *const u8 as *const libc::c_char);
    }
    let mut opt: i32 = 1 as i32;
    if setsockopt(
        (*this).sock,
        0xffff as i32,
        0x4 as i32,
        &mut opt as *mut i32 as *mut libc::c_char as *const libc::c_void,
        ::core::mem::size_of::<i32>() as libc::c_ulong as socklen_t,
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
    return this;
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
    let mut sock: sock_t = accept((*this).sock, 0 as *mut sockaddr, 0 as *mut socklen_t);
    if sock == -(1 as i32) {
        return 0 as *mut Socket;
    }
    let mut con: *mut Socket = MemAlloc(::core::mem::size_of::<Socket>() as usize) as *mut Socket;
    (*con).type_0 = SocketType_TCP;
    (*con).sock = sock;
    if !Socket_SetNonblocking((*con).sock) {
        Fatal(
            b"Socket_Accept: failed to set socket to non-blocking\0" as *const u8
                as *const libc::c_char,
        );
    }
    return con;
}
#[no_mangle]
pub unsafe extern "C" fn Socket_Bind(mut this: *mut Socket, mut port: i32) {
    let mut addr: sockaddr_in = sockaddr_in {
        sin_len: 0,
        sin_family: 0,
        sin_port: 0,
        sin_addr: in_addr { s_addr: 0 },
        sin_zero: [0; 8],
    };
    MemSet(
        &mut addr as *mut sockaddr_in as *mut libc::c_void,
        0 as i32,
        ::core::mem::size_of::<sockaddr_in>() as usize,
    );
    addr.sin_family = 2 as i32 as sa_family_t;
    addr.sin_port = (if 0 != 0 {
        ((port as u16 as u32 & 0xff00 as u32) >> 8 as i32
            | (port as u16 as u32 & 0xff as u32) << 8 as i32) as __u16 as i32
    } else {
        _OSSwapInt16(port as u16) as i32
    }) as __u16;
    addr.sin_addr.s_addr = if 0 != 0 {
        (0 as i32 as u_i32_t & 0xff000000 as u32) >> 24 as i32
            | (0 as i32 as u_i32_t & 0xff0000 as u32) >> 8 as i32
            | (0 as i32 as u_i32_t & 0xff00 as u32) << 8 as i32
            | (0 as i32 as u_i32_t & 0xff as u32) << 24 as i32
    } else {
        _OSSwapInt32(0 as i32 as u_i32_t)
    };
    if bind(
        (*this).sock,
        &mut addr as *mut sockaddr_in as *const sockaddr,
        ::core::mem::size_of::<sockaddr_in>() as usize as socklen_t,
    ) == -(1 as i32)
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
    if listen((*this).sock, 1 as i32) == -(1 as i32) {
        Fatal(b"Socket_Listen: failed to listen\0" as *const u8 as *const libc::c_char);
    }
}
#[no_mangle]
pub unsafe extern "C" fn Socket_Read(mut this: *mut Socket) -> *const libc::c_char {
    let mut bytes: i32 = Socket_Receive(
        (*this).sock,
        ((*this).buffer).as_mut_ptr() as *mut libc::c_void,
        ::core::mem::size_of::<[libc::c_char; 2048]>() as libc::c_ulong as i32,
    );
    if bytes == -(1 as i32) {
        if std::io::Error::last_os_error().raw_os_error().unwrap_or(0) == 35 as i32 {
            return 0 as *const libc::c_char;
        }
        Fatal(b"Socket_Read: failed to read from socket\0" as *const u8 as *const libc::c_char);
    }
    if bytes == 0 as i32 {
        return 0 as *const libc::c_char;
    }
    (*this).buffer[bytes as usize] = 0 as i32 as libc::c_char;
    return ((*this).buffer).as_mut_ptr() as *const libc::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn Socket_ReadBytes(mut this: *mut Socket) -> *mut Bytes {
    let mut bytes: i32 = Socket_Receive(
        (*this).sock,
        ((*this).buffer).as_mut_ptr() as *mut libc::c_void,
        ::core::mem::size_of::<[libc::c_char; 2048]>() as libc::c_ulong as i32,
    );
    if bytes == -(1 as i32) {
        if std::io::Error::last_os_error().raw_os_error().unwrap_or(0) == 35 as i32 {
            return 0 as *mut Bytes;
        }
        Fatal(b"Socket_ReadRaw: failed to read from socket\0" as *const u8 as *const libc::c_char);
    }
    if bytes == 0 as i32 {
        return 0 as *mut Bytes;
    }
    let mut data: *mut Bytes = Bytes_Create(bytes as u32);
    Bytes_Write(
        data,
        ((*this).buffer).as_mut_ptr() as *const libc::c_void,
        bytes as u32,
    );
    return data;
}
#[no_mangle]
pub unsafe extern "C" fn Socket_ReceiveFrom(
    mut this: *mut Socket,
    mut data: *mut libc::c_void,
    mut len: usize,
) -> i32 {
    MemZero(data, len);
    let mut addrSize: socklen_t = 0;
    let mut bytes: i32 = recvfrom(
        (*this).sock,
        data,
        len,
        0 as i32,
        &mut (*this).addrRecv as *mut sockaddr_in as *mut sockaddr,
        &mut addrSize,
    ) as i32;
    if bytes == -(1 as i32) {
        if std::io::Error::last_os_error().raw_os_error().unwrap_or(0) == 35 as i32 {
            return 0 as i32;
        }
        return -(1 as i32);
    }
    return bytes;
}
#[no_mangle]
pub unsafe extern "C" fn Socket_GetAddress(mut this: *mut Socket) -> *const libc::c_char {
    return StrFormat(
        b"%s:%d\0" as *const u8 as *const libc::c_char,
        inet_ntoa((*this).addrRecv.sin_addr),
        (if 0 != 0 {
            (((*this).addrRecv.sin_port as u32 & 0xff00 as u32) >> 8 as i32
                | ((*this).addrRecv.sin_port as u32 & 0xff as u32) << 8 as i32) as __u16
                as i32
        } else {
            _OSSwapInt16((*this).addrRecv.sin_port) as i32
        }) as __u16 as i32,
    );
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
    let mut port: *const libc::c_char = StrSubStr(colon.offset(1), addr.offset(strlen(addr) as isize));
    (*this).addrSend.sin_family = 2 as i32 as sa_family_t;
    (*this).addrSend.sin_port = (if 0 != 0 {
        ((strtol(port, 0 as *mut *mut libc::c_char, 0 as i32) as u16 as u32 & 0xff00 as u32)
            >> 8 as i32
            | (strtol(port, 0 as *mut *mut libc::c_char, 0 as i32) as u16 as u32 & 0xff as u32)
                << 8 as i32) as __u16 as i32
    } else {
        _OSSwapInt16(strtol(port, 0 as *mut *mut libc::c_char, 0 as i32) as u16) as i32
    }) as __u16;
    if inet_aton(ip, &mut (*this).addrSend.sin_addr) == 0 as i32 {
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
    let mut bytes: i32 = sendto(
        (*this).sock,
        data,
        len,
        0 as i32,
        &mut (*this).addrSend as *mut sockaddr_in as *const sockaddr,
        ::core::mem::size_of::<sockaddr_in>() as usize as socklen_t,
    ) as i32;
    if bytes == -(1 as i32) {
        return -(1 as i32);
    }
    return bytes;
}
#[no_mangle]
pub unsafe extern "C" fn Socket_Write(mut this: *mut Socket, mut msg: *const libc::c_char) {
    if Socket_Send((*this).sock, msg as *const libc::c_void, StrLen(msg) as i32) == -(1 as i32) {
        Fatal(b"Socket_Write: failed to write to socket\0" as *const u8 as *const libc::c_char);
    }
}
#[no_mangle]
pub unsafe extern "C" fn Socket_WriteBytes(mut this: *mut Socket, mut msg: *mut Bytes) {
    if Socket_Send((*this).sock, Bytes_GetData(msg), Bytes_GetSize(msg) as i32) == -(1 as i32) {
        Fatal(b"Socket_WriteRaw: failed to write to socket\0" as *const u8 as *const libc::c_char);
    }
}
