mod checks;
mod leak;

pub(super) use super::{apps, gateway, http, kernel, kit};

use gateway::Gateway;
use kernel::Kernel;

pub fn run() {
    let Some((bin, p0)) = kit::kit() else {
        eprintln!("SKIP: canonical kit required");
        return;
    };
    let token = [0x42u8; 32];
    let cap_a = "edge-cap-a-unique";
    let cap_b = "edge-cap-b-unique";
    let kernel = Kernel::start(&bin, &p0, token);
    let mut gw = Gateway::start(
        &kernel.url(),
        token,
        apps::two_apps(cap_a, cap_b),
        16 * 1024 * 1024,
    );
    checks::denials(&gw, cap_a);
    checks::cross_app(&gw, cap_a);
    leak::assert_no_leak(&mut gw, token, cap_a, cap_b);
}
