use run_script::ScriptOptions;
use rustcracker::components::machine::MachineError;

use crate::error::{VmManageError, VmManageResult};

pub fn set_network_for_one_machine(count: u32) -> VmManageResult<()> {
    // set up network
    let (code, _output, error) = run_script::run_script!(
        r#"
        TAP_DEV="tap$1"
        HOST_IFACE="eth$1"
        TAP_IP="172.16.0.1"
        MASK_SHORT="/30"

        # Setup network interface
        sudo ip link del "$TAP_DEV" 2> /dev/null || true
        sudo ip tuntap add dev "$TAP_DEV" mode tap
        sudo ip addr add "${TAP_IP}${MASK_SHORT}" dev "$TAP_DEV"
        sudo ip link set dev "$TAP_DEV" up

        # Enable ip forwarding
        sudo sh -c "echo 1 > /proc/sys/net/ipv4/ip_forward"

        # Set up microVM internet access
        sudo iptables -t nat -D POSTROUTING -o "$HOST_IFACE" -j MASQUERADE || true
        sudo iptables -D FORWARD -m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT \
            || true
        sudo iptables -D FORWARD -i "$TAP_DEV" -o "$HOST_IFACE" -j ACCEPT || true
        sudo iptables -t nat -A POSTROUTING -o "$HOST_IFACE" -j MASQUERADE
        sudo iptables -I FORWARD 1 -m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT
        sudo iptables -I FORWARD 1 -i "$TAP_DEV" -o "$HOST_IFACE" -j ACCEPT
        "#,
        &vec![format!("{}", count.to_string())],
        &ScriptOptions::new()
    ).unwrap();
    // if networking is configured successfully, then run the machine `name{id}``
    if !(code == 0 && error == "") {
        log::error!(target: "main", "Fail to set up network");
        return Err(VmManageError::NetworkError(format!("Fail to set up network: {}",error)));
    }
    Ok(())
}