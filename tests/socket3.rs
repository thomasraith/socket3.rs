extern crate socket3;

use socket3::raw::*;
use std::io::Write;

#[test]
fn raw_socket_ipv6_test() {
    let socket = Socket::new(Domain::IPV6, Protocol::RAW);

    if let Err(error) = socket {
        let error_code = error.raw_os_error().unwrap();

        //Checks is the error means missing permissions (test requires root privileges)
        assert_eq!(error_code, 1);
        let _ = writeln!(&mut std::io::stderr(), "############################################");
        let _ = writeln!(&mut std::io::stderr(), "# ✘  Missing root privileges: Test aborted #");
        let _ = writeln!(&mut std::io::stderr(), "############################################");
        return;
    }

    assert!(socket.is_ok());
}
