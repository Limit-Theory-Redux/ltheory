use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
extern "C" {
    pub type Bytes;
    fn Bytes_GetSize(_: *mut Bytes) -> uint32;
    fn strlen(_: *const libc::c_char) -> libc::c_ulong;
    fn strstr(_: *const libc::c_char, _: *const libc::c_char) -> *mut libc::c_char;
    fn Bytes_GetData(_: *mut Bytes) -> *mut libc::c_void;
    fn Fatal(_: cstr, _: ...);
    fn Bytes_Create(len: uint32) -> *mut Bytes;
    fn Bytes_Write(_: *mut Bytes, data: *const libc::c_void, len: uint32);
    fn strtol(
        _: *const libc::c_char,
        _: *mut *mut libc::c_char,
        _: libc::c_int,
    ) -> libc::c_long;
    fn vsnprintf(
        _: *mut libc::c_char,
        _: libc::size_t,
        _: *const libc::c_char,
        _: __builtin_va_list,
    ) -> libc::c_int;
    fn error() -> *mut libc::c_int;
    fn fcntl(_: libc::c_int, _: libc::c_int, _: ...) -> libc::c_int;
    fn close(_: libc::c_int) -> libc::c_int;
    fn read(_: libc::c_int, _: *mut libc::c_void, _: libc::size_t) -> libc::ssize_t;
    fn write(__fd: libc::c_int, __buf: *const libc::c_void, __nbyte: libc::size_t) -> libc::ssize_t;
    fn accept(_: libc::c_int, _: *mut sockaddr, _: *mut socklen_t) -> libc::c_int;
    fn bind(_: libc::c_int, _: *const sockaddr, _: socklen_t) -> libc::c_int;
    fn listen(_: libc::c_int, _: libc::c_int) -> libc::c_int;
    fn recvfrom(
        _: libc::c_int,
        _: *mut libc::c_void,
        _: libc::size_t,
        _: libc::c_int,
        _: *mut sockaddr,
        _: *mut socklen_t,
    ) -> libc::ssize_t;
    fn sendto(
        _: libc::c_int,
        _: *const libc::c_void,
        _: libc::size_t,
        _: libc::c_int,
        _: *const sockaddr,
        _: socklen_t,
    ) -> libc::ssize_t;
    fn setsockopt(
        _: libc::c_int,
        _: libc::c_int,
        _: libc::c_int,
        _: *const libc::c_void,
        _: socklen_t,
    ) -> libc::c_int;
    fn socket(_: libc::c_int, _: libc::c_int, _: libc::c_int) -> libc::c_int;
    fn inet_ntoa(_: in_addr) -> *mut libc::c_char;
    fn inet_aton(_: *const libc::c_char, _: *mut in_addr) -> libc::c_int;
}
pub type __builtin_va_list = *mut libc::c_char;
pub type int32_t = libc::c_int;
pub type uint16_t = libc::c_ushort;
pub type uint32_t = libc::c_uint;
pub type __uint8_t = libc::c_uchar;
pub type __uint16_t = libc::c_ushort;
pub type __uint32_t = libc::c_uint;
pub type __darwin_socklen_t = __uint32_t;
pub type u_int32_t = libc::c_uint;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;
pub type uint16 = uint16_t;
pub type uint32 = uint32_t;
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
    pub sin_len: __uint8_t,
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
pub type in_addr_t = __uint32_t;
pub type in_port_t = __uint16_t;
pub type sa_family_t = __uint8_t;
pub type sock_t = libc::c_int;
pub type SocketType = int32;
pub type socklen_t = __darwin_socklen_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sockaddr {
    pub sa_len: __uint8_t,
    pub sa_family: sa_family_t,
    pub sa_data: [libc::c_char; 14],
}
pub type va_list = __builtin_va_list;
#[inline]
unsafe extern "C" fn _OSSwapInt32(mut _data: uint32_t) -> uint32_t {
    _data = _data.swap_bytes();
    return _data;
}
#[inline]
unsafe extern "C" fn _OSSwapInt16(mut _data: uint16_t) -> uint16_t {
    return ((_data as libc::c_int) << 8 as libc::c_int
        | _data as libc::c_int >> 8 as libc::c_int) as uint16_t;
}

#[no_mangle]
pub static mut SocketType_None: SocketType = 0 as libc::c_int;
#[no_mangle]
pub static mut SocketType_UDP: SocketType = 0x1 as libc::c_int;
#[no_mangle]
pub static mut SocketType_TCP: SocketType = 0x2 as libc::c_int;


#[inline]
unsafe extern "C" fn StrFind(mut s: cstr, mut sub: cstr) -> cstr {
    return strstr(s, sub) as cstr;
}
#[inline]
unsafe extern "C" fn StrSubStr(mut begin: cstr, mut end: cstr) -> cstr {
    let mut len: libc::size_t = end.offset_from(begin) as libc::c_long as libc::size_t;
    let mut result: *mut libc::c_char = StrAlloc(
        len.wrapping_add(1 as libc::size_t),
    );
    let mut pResult: *mut libc::c_char = result;
    while begin != end {
        let fresh0 = begin;
        begin = begin.offset(1);
        let fresh1 = pResult;
        pResult = pResult.offset(1);
        *fresh1 = *fresh0;
    }
    *result.offset(len as isize) = 0 as libc::c_int as libc::c_char;
    return result as cstr;
}
#[inline]
unsafe extern "C" fn Socket_Cleanup(mut self_0: sock_t) {
    if self_0 != -(1 as libc::c_int) {
        close(self_0);
    }
}
#[inline]
unsafe extern "C" fn Socket_Receive(
    mut self_0: sock_t,
    mut buf: *mut libc::c_void,
    mut len: libc::c_int,
) -> libc::c_int {
    return read(self_0, buf, len as usize) as libc::c_int;
}
#[inline]
unsafe extern "C" fn Socket_Send(
    mut self_0: sock_t,
    mut buf: *const libc::c_void,
    mut len: libc::c_int,
) -> libc::c_int {
    return write(self_0, buf, len as usize) as libc::c_int;
}
#[inline]
unsafe extern "C" fn Socket_SetNonblocking(mut self_0: sock_t) -> bool {
    return fcntl(
        self_0,
        4 as libc::c_int,
        fcntl(self_0, 3 as libc::c_int, 0 as libc::c_int) | 0x4 as libc::c_int,
    ) >= 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn Socket_Create(mut type_0: SocketType) -> *mut Socket {
    if type_0 != SocketType_UDP && type_0 != SocketType_TCP {
        Fatal(
            b"Socket_Create: socket type must be either SocketType_TCP or SocketType_UDP\0"
                as *const u8 as *const libc::c_char,
        );
    }
    let mut self_0: *mut Socket = MemAlloc(
        ::core::mem::size_of::<Socket>() as usize,
    ) as *mut Socket;
    (*self_0).type_0 = type_0;
    (*self_0)
        .sock = socket(
        2 as libc::c_int,
        if type_0 == SocketType_UDP { 2 as libc::c_int } else { 1 as libc::c_int },
        0 as libc::c_int,
    );
    if (*self_0).sock == -(1 as libc::c_int) {
        Fatal(
            b"Socket_Create: failed to open socket\0" as *const u8 as *const libc::c_char,
        );
    }
    let mut opt: libc::c_int = 1 as libc::c_int;
    if setsockopt(
        (*self_0).sock,
        0xffff as libc::c_int,
        0x4 as libc::c_int,
        &mut opt as *mut libc::c_int as *mut libc::c_char as *const libc::c_void,
        ::core::mem::size_of::<libc::c_int>() as libc::c_ulong as socklen_t,
    ) != 0
    {
        Fatal(
            b"Socket_Create: failed to set socket to reusable\0" as *const u8
                as *const libc::c_char,
        );
    }
    if !Socket_SetNonblocking((*self_0).sock) {
        Fatal(
            b"Socket_Create: failed to set socket to non-blocking\0" as *const u8
                as *const libc::c_char,
        );
    }
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn Socket_Free(mut self_0: *mut Socket) {
    Socket_Cleanup((*self_0).sock);
    MemFree(self_0 as *const libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn Socket_Accept(mut self_0: *mut Socket) -> *mut Socket {
    if (*self_0).type_0 != SocketType_TCP {
        Fatal(
            b"Socket_Accept: can only accept connections on TCP sockets\0" as *const u8
                as *const libc::c_char,
        );
    }
    let mut sock: sock_t = accept(
        (*self_0).sock,
        0 as *mut sockaddr,
        0 as *mut socklen_t,
    );
    if sock == -(1 as libc::c_int) {
        return 0 as *mut Socket;
    }
    let mut con: *mut Socket = MemAlloc(
        ::core::mem::size_of::<Socket>() as usize,
    ) as *mut Socket;
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
pub unsafe extern "C" fn Socket_Bind(mut self_0: *mut Socket, mut port: libc::c_int) {
    let mut addr: sockaddr_in = sockaddr_in {
        sin_len: 0,
        sin_family: 0,
        sin_port: 0,
        sin_addr: in_addr { s_addr: 0 },
        sin_zero: [0; 8],
    };
    MemSet(
        &mut addr as *mut sockaddr_in as *mut libc::c_void,
        0 as libc::c_int,
        ::core::mem::size_of::<sockaddr_in>() as usize,
    );
    addr.sin_family = 2 as libc::c_int as sa_family_t;
    addr
        .sin_port = (if 0 != 0 {
        ((port as uint16 as libc::c_uint & 0xff00 as libc::c_uint) >> 8 as libc::c_int
            | (port as uint16 as libc::c_uint & 0xff as libc::c_uint)
                << 8 as libc::c_int) as __uint16_t as libc::c_int
    } else {
        _OSSwapInt16(port as uint16) as libc::c_int
    }) as __uint16_t;
    addr
        .sin_addr
        .s_addr = if 0 != 0 {
        (0 as libc::c_int as u_int32_t & 0xff000000 as libc::c_uint) >> 24 as libc::c_int
            | (0 as libc::c_int as u_int32_t & 0xff0000 as libc::c_uint)
                >> 8 as libc::c_int
            | (0 as libc::c_int as u_int32_t & 0xff00 as libc::c_uint)
                << 8 as libc::c_int
            | (0 as libc::c_int as u_int32_t & 0xff as libc::c_uint) << 24 as libc::c_int
    } else {
        _OSSwapInt32(0 as libc::c_int as u_int32_t)
    };
    if bind(
        (*self_0).sock,
        &mut addr as *mut sockaddr_in as *const sockaddr,
        ::core::mem::size_of::<sockaddr_in>() as usize as socklen_t,
    ) == -(1 as libc::c_int)
    {
        Fatal(
            b"Socket_Bind: failed to bind socket\0" as *const u8 as *const libc::c_char,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn Socket_Listen(mut self_0: *mut Socket) {
    if (*self_0).type_0 != SocketType_TCP {
        Fatal(
            b"Socket_Listen: can only listen for connections on TCP sockets\0"
                as *const u8 as *const libc::c_char,
        );
    }
    if listen((*self_0).sock, 1 as libc::c_int) == -(1 as libc::c_int) {
        Fatal(b"Socket_Listen: failed to listen\0" as *const u8 as *const libc::c_char);
    }
}
#[no_mangle]
pub unsafe extern "C" fn Socket_Read(mut self_0: *mut Socket) -> cstr {
    let mut bytes: libc::c_int = Socket_Receive(
        (*self_0).sock,
        ((*self_0).buffer).as_mut_ptr() as *mut libc::c_void,
        ::core::mem::size_of::<[libc::c_char; 2048]>() as libc::c_ulong as libc::c_int,
    );
    if bytes == -(1 as libc::c_int) {
        if *error() == 35 as libc::c_int {
            return 0 as cstr;
        }
        Fatal(
            b"Socket_Read: failed to read from socket\0" as *const u8
                as *const libc::c_char,
        );
    }
    if bytes == 0 as libc::c_int {
        return 0 as cstr;
    }
    (*self_0).buffer[bytes as usize] = 0 as libc::c_int as libc::c_char;
    return ((*self_0).buffer).as_mut_ptr() as cstr;
}
#[no_mangle]
pub unsafe extern "C" fn Socket_ReadBytes(mut self_0: *mut Socket) -> *mut Bytes {
    let mut bytes: libc::c_int = Socket_Receive(
        (*self_0).sock,
        ((*self_0).buffer).as_mut_ptr() as *mut libc::c_void,
        ::core::mem::size_of::<[libc::c_char; 2048]>() as libc::c_ulong as libc::c_int,
    );
    if bytes == -(1 as libc::c_int) {
        if *error() == 35 as libc::c_int {
            return 0 as *mut Bytes;
        }
        Fatal(
            b"Socket_ReadRaw: failed to read from socket\0" as *const u8
                as *const libc::c_char,
        );
    }
    if bytes == 0 as libc::c_int {
        return 0 as *mut Bytes;
    }
    let mut data: *mut Bytes = Bytes_Create(bytes as uint32);
    Bytes_Write(
        data,
        ((*self_0).buffer).as_mut_ptr() as *const libc::c_void,
        bytes as uint32,
    );
    return data;
}
#[no_mangle]
pub unsafe extern "C" fn Socket_ReceiveFrom(
    mut self_0: *mut Socket,
    mut data: *mut libc::c_void,
    mut len: libc::size_t,
) -> libc::c_int {
    MemZero(data, len);
    let mut addrSize: socklen_t = 0;
    let mut bytes: libc::c_int = recvfrom(
        (*self_0).sock,
        data,
        len,
        0 as libc::c_int,
        &mut (*self_0).addrRecv as *mut sockaddr_in as *mut sockaddr,
        &mut addrSize,
    ) as libc::c_int;
    if bytes == -(1 as libc::c_int) {
        if *error() == 35 as libc::c_int {
            return 0 as libc::c_int;
        }
        return -(1 as libc::c_int);
    }
    return bytes;
}
#[no_mangle]
pub unsafe extern "C" fn Socket_GetAddress(mut self_0: *mut Socket) -> cstr {
    return StrFormat(
        b"%s:%d\0" as *const u8 as *const libc::c_char,
        inet_ntoa((*self_0).addrRecv.sin_addr),
        (if 0 != 0 {
            (((*self_0).addrRecv.sin_port as libc::c_uint & 0xff00 as libc::c_uint)
                >> 8 as libc::c_int
                | ((*self_0).addrRecv.sin_port as libc::c_uint & 0xff as libc::c_uint)
                    << 8 as libc::c_int) as __uint16_t as libc::c_int
        } else {
            _OSSwapInt16((*self_0).addrRecv.sin_port) as libc::c_int
        }) as __uint16_t as libc::c_int,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Socket_SetAddress(mut self_0: *mut Socket, mut addr: cstr) {
    let mut colon: cstr = StrFind(addr, b":\0" as *const u8 as *const libc::c_char);
    if colon.is_null() {
        Fatal(
            b"Socket_SetReceiver: address must be in format a.b.c.d:port format\0"
                as *const u8 as *const libc::c_char,
        );
    }
    let mut ip: cstr = StrSubStr(addr, colon);
    let mut port: cstr = StrSubStr(
        colon.offset(1),
        addr.offset(strlen(addr) as isize),
    );
    (*self_0).addrSend.sin_family = 2 as libc::c_int as sa_family_t;
    (*self_0)
        .addrSend
        .sin_port = (if 0 != 0 {
        ((strtol(port, 0 as *mut *mut libc::c_char, 0 as libc::c_int) as uint16
            as libc::c_uint & 0xff00 as libc::c_uint) >> 8 as libc::c_int
            | (strtol(port, 0 as *mut *mut libc::c_char, 0 as libc::c_int) as uint16
                as libc::c_uint & 0xff as libc::c_uint) << 8 as libc::c_int)
            as __uint16_t as libc::c_int
    } else {
        _OSSwapInt16(
            strtol(port, 0 as *mut *mut libc::c_char, 0 as libc::c_int) as uint16,
        ) as libc::c_int
    }) as __uint16_t;
    if inet_aton(ip, &mut (*self_0).addrSend.sin_addr) == 0 as libc::c_int {
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
    mut self_0: *mut Socket,
    mut data: *const libc::c_void,
    mut len: libc::size_t,
) -> libc::c_int {
    let mut bytes: libc::c_int = sendto(
        (*self_0).sock,
        data,
        len,
        0 as libc::c_int,
        &mut (*self_0).addrSend as *mut sockaddr_in as *const sockaddr,
        ::core::mem::size_of::<sockaddr_in>() as usize as socklen_t,
    ) as libc::c_int;
    if bytes == -(1 as libc::c_int) {
        return -(1 as libc::c_int);
    }
    return bytes;
}
#[no_mangle]
pub unsafe extern "C" fn Socket_Write(mut self_0: *mut Socket, mut msg: cstr) {
    if Socket_Send(
        (*self_0).sock,
        msg as *const libc::c_void,
        StrLen(msg) as libc::c_int,
    ) == -(1 as libc::c_int)
    {
        Fatal(
            b"Socket_Write: failed to write to socket\0" as *const u8
                as *const libc::c_char,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn Socket_WriteBytes(
    mut self_0: *mut Socket,
    mut msg: *mut Bytes,
) {
    if Socket_Send((*self_0).sock, Bytes_GetData(msg), Bytes_GetSize(msg) as libc::c_int)
        == -(1 as libc::c_int)
    {
        Fatal(
            b"Socket_WriteRaw: failed to write to socket\0" as *const u8
                as *const libc::c_char,
        );
    }
}
