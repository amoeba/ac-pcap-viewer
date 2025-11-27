use super::reader::BinaryReader;
use anyhow::{bail, Result};
use serde::Serialize;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PacketHeaderFlags: u32 {
        const NONE = 0x00000000;
        const RETRANSMISSION = 0x00000001;
        const ENCRYPTED_CHECKSUM = 0x00000002;
        const BLOB_FRAGMENTS = 0x00000004;
        const SERVER_SWITCH = 0x00000100;
        const LOGON_SERVER_ADDR = 0x00000200;
        const EMPTY_HEADER1 = 0x00000400;
        const REFERRAL = 0x00000800;
        const REQUEST_RETRANSMIT = 0x00001000;
        const REJECT_RETRANSMIT = 0x00002000;
        const ACK_SEQUENCE = 0x00004000;
        const DISCONNECT = 0x00008000;
        const LOGIN_REQUEST = 0x00010000;
        const WORLD_LOGIN_REQUEST = 0x00020000;
        const CONNECT_REQUEST = 0x00040000;
        const CONNECT_RESPONSE = 0x00080000;
        const NET_ERROR = 0x00100000;
        const NET_ERROR_DISCONNECT = 0x00200000;
        const CICMD_COMMAND = 0x00400000;
        const TIME_SYNC = 0x01000000;
        const ECHO_REQUEST = 0x02000000;
        const ECHO_RESPONSE = 0x04000000;
        const FLOW = 0x08000000;
    }
}

impl Serialize for PacketHeaderFlags {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut names = Vec::new();
        if self.contains(PacketHeaderFlags::RETRANSMISSION) {
            names.push("Retransmission");
        }
        if self.contains(PacketHeaderFlags::ENCRYPTED_CHECKSUM) {
            names.push("EncryptedChecksum");
        }
        if self.contains(PacketHeaderFlags::BLOB_FRAGMENTS) {
            names.push("BlobFragments");
        }
        if self.contains(PacketHeaderFlags::SERVER_SWITCH) {
            names.push("ServerSwitch");
        }
        if self.contains(PacketHeaderFlags::LOGON_SERVER_ADDR) {
            names.push("LogonServerAddr");
        }
        if self.contains(PacketHeaderFlags::EMPTY_HEADER1) {
            names.push("EmptyHeader1");
        }
        if self.contains(PacketHeaderFlags::REFERRAL) {
            names.push("Referral");
        }
        if self.contains(PacketHeaderFlags::REQUEST_RETRANSMIT) {
            names.push("RequestRetransmit");
        }
        if self.contains(PacketHeaderFlags::REJECT_RETRANSMIT) {
            names.push("RejectRetransmit");
        }
        if self.contains(PacketHeaderFlags::ACK_SEQUENCE) {
            names.push("AckSequence");
        }
        if self.contains(PacketHeaderFlags::DISCONNECT) {
            names.push("Disconnect");
        }
        if self.contains(PacketHeaderFlags::LOGIN_REQUEST) {
            names.push("LoginRequest");
        }
        if self.contains(PacketHeaderFlags::WORLD_LOGIN_REQUEST) {
            names.push("WorldLoginRequest");
        }
        if self.contains(PacketHeaderFlags::CONNECT_REQUEST) {
            names.push("ConnectRequest");
        }
        if self.contains(PacketHeaderFlags::CONNECT_RESPONSE) {
            names.push("ConnectResponse");
        }
        if self.contains(PacketHeaderFlags::NET_ERROR) {
            names.push("NetError");
        }
        if self.contains(PacketHeaderFlags::NET_ERROR_DISCONNECT) {
            names.push("NetErrorDisconnect");
        }
        if self.contains(PacketHeaderFlags::CICMD_COMMAND) {
            names.push("CICMDCommand");
        }
        if self.contains(PacketHeaderFlags::TIME_SYNC) {
            names.push("TimeSync");
        }
        if self.contains(PacketHeaderFlags::ECHO_REQUEST) {
            names.push("EchoRequest");
        }
        if self.contains(PacketHeaderFlags::ECHO_RESPONSE) {
            names.push("EchoResponse");
        }
        if self.contains(PacketHeaderFlags::FLOW) {
            names.push("Flow");
        }

        if names.is_empty() {
            serializer.serialize_str("None")
        } else {
            serializer.serialize_str(&names.join(", "))
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerSwitchHeader {
    // TODO: fill in fields
}

#[derive(Debug, Clone, Serialize)]
pub struct LogonServerAddrHeader {
    // TODO: fill in fields
}

#[derive(Debug, Clone, Serialize)]
pub struct ReferralHeader {
    // TODO: fill in fields
}

#[derive(Debug, Clone, Serialize)]
pub struct AckSequenceHeader {
    pub sequence: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoginRequestHeader {
    // TODO: fill in fields
}

#[derive(Debug, Clone, Serialize)]
pub struct WorldLoginRequestHeader {
    // TODO: fill in fields
}

#[derive(Debug, Clone, Serialize)]
pub struct ConnectRequestHeader {
    // TODO: fill in fields
}

#[derive(Debug, Clone, Serialize)]
pub struct ConnectResponseHeader {
    // TODO: fill in fields
}

#[derive(Debug, Clone, Serialize)]
pub struct NetErrorHeader {
    // TODO: fill in fields
}

#[derive(Debug, Clone, Serialize)]
pub struct NetErrorDisconnectHeader {
    // TODO: fill in fields
}

#[derive(Debug, Clone, Serialize)]
pub struct CICMDCommandHeader {
    // TODO: fill in fields
}

#[derive(Debug, Clone, Serialize)]
pub struct TimeSyncHeader {
    #[serde(rename = "Time")]
    pub time: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct EchoRequestHeader {
    #[serde(rename = "LocalTime")]
    pub local_time: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct EchoResponseHeader {
    #[serde(rename = "LocalTime")]
    pub local_time: f32,
    #[serde(rename = "HoldingTime")]
    pub holding_time: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct FlowHeader {
    #[serde(rename = "DataReceived")]
    pub data_received: u32,
    #[serde(rename = "Interval")]
    pub interval: u16,
}

#[derive(Debug, Clone, Serialize)]
pub struct RequestRetransmitHeader {
    #[serde(rename = "SequenceIds")]
    pub sequence_ids: Vec<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct PacketHeader {
    #[serde(rename = "Sequence")]
    pub sequence: u32,
    #[serde(rename = "Flags")]
    pub flags: PacketHeaderFlags,
    #[serde(rename = "Checksum")]
    pub checksum: u32,
    #[serde(rename = "Id")]
    pub id: u16,
    #[serde(rename = "Time")]
    pub time: u16,
    #[serde(rename = "Size")]
    pub size: u16,
    #[serde(rename = "Iteration")]
    pub iteration: u16,

    // Optional headers based on flags
    #[serde(rename = "ServerSwitch")]
    pub server_switch: Option<ServerSwitchHeader>,
    #[serde(rename = "LogonServerAddr")]
    pub logon_server_addr: Option<LogonServerAddrHeader>,
    #[serde(rename = "Referral")]
    pub referral: Option<ReferralHeader>,
    #[serde(rename = "AckSequence")]
    pub ack_sequence: Option<AckSequenceHeader>,
    #[serde(rename = "LoginRequest")]
    pub login_request: Option<LoginRequestHeader>,
    #[serde(rename = "WorldLoginRequest")]
    pub world_login_request: Option<WorldLoginRequestHeader>,
    #[serde(rename = "ConnectRequest")]
    pub connect_request: Option<ConnectRequestHeader>,
    #[serde(rename = "ConnectResponse")]
    pub connect_response: Option<ConnectResponseHeader>,
    #[serde(rename = "NetError")]
    pub net_error: Option<NetErrorHeader>,
    #[serde(rename = "NetErrorDisconnect")]
    pub net_error_disconnect: Option<NetErrorDisconnectHeader>,
    #[serde(rename = "CICMDCommand")]
    pub cicmd_command: Option<CICMDCommandHeader>,
    #[serde(rename = "TimeSync")]
    pub time_sync: Option<TimeSyncHeader>,
    #[serde(rename = "EchoRequest")]
    pub echo_request: Option<EchoRequestHeader>,
    #[serde(rename = "EchoResponse")]
    pub echo_response: Option<EchoResponseHeader>,
    #[serde(rename = "Flow")]
    pub flow: Option<FlowHeader>,
    #[serde(rename = "RequestRetransmit")]
    pub request_retransmit: Option<RequestRetransmitHeader>,
}

impl PacketHeader {
    pub const BASE_SIZE: usize = 20;

    pub fn parse(reader: &mut BinaryReader) -> Result<Self> {
        let sequence = reader.read_u32()?;
        let flags_raw = reader.read_u32()?;
        let flags = PacketHeaderFlags::from_bits_truncate(flags_raw);
        let checksum = reader.read_u32()?;
        let id = reader.read_u16()?;
        let time = reader.read_u16()?;
        let size = reader.read_u16()?;
        let iteration = reader.read_u16()?;

        let mut header = Self {
            sequence,
            flags,
            checksum,
            id,
            time,
            size,
            iteration,
            server_switch: None,
            logon_server_addr: None,
            referral: None,
            ack_sequence: None,
            login_request: None,
            world_login_request: None,
            connect_request: None,
            connect_response: None,
            net_error: None,
            net_error_disconnect: None,
            cicmd_command: None,
            time_sync: None,
            echo_request: None,
            echo_response: None,
            flow: None,
            request_retransmit: None,
        };

        // Parse optional headers in the correct order (matching C# implementation)
        if flags.contains(PacketHeaderFlags::SERVER_SWITCH) {
            header.server_switch = Some(ServerSwitchHeader {});
            bail!("ServerSwitch parsing not implemented");
        }

        if flags.contains(PacketHeaderFlags::LOGON_SERVER_ADDR) {
            header.logon_server_addr = Some(LogonServerAddrHeader {});
            bail!("LogonServerAddr parsing not implemented");
        }

        if flags.contains(PacketHeaderFlags::REQUEST_RETRANSMIT) {
            let num = reader.read_u32()?;
            let mut sequence_ids = Vec::new();
            for _ in 0..num {
                sequence_ids.push(reader.read_u32()?);
            }
            header.request_retransmit = Some(RequestRetransmitHeader { sequence_ids });
        }

        if flags.contains(PacketHeaderFlags::REFERRAL) {
            header.referral = Some(ReferralHeader {});
            bail!("Referral parsing not implemented");
        }

        if flags.contains(PacketHeaderFlags::ACK_SEQUENCE) {
            let seq = reader.read_u32()?;
            header.ack_sequence = Some(AckSequenceHeader { sequence: seq });
        }

        if flags.contains(PacketHeaderFlags::LOGIN_REQUEST) {
            header.login_request = Some(LoginRequestHeader {});
            bail!("LoginRequest parsing not implemented");
        }

        if flags.contains(PacketHeaderFlags::WORLD_LOGIN_REQUEST) {
            header.world_login_request = Some(WorldLoginRequestHeader {});
            bail!("WorldLoginRequest parsing not implemented");
        }

        if flags.contains(PacketHeaderFlags::CONNECT_REQUEST) {
            header.connect_request = Some(ConnectRequestHeader {});
            bail!("ConnectRequest parsing not implemented");
        }

        if flags.contains(PacketHeaderFlags::CONNECT_RESPONSE) {
            header.connect_response = Some(ConnectResponseHeader {});
            bail!("ConnectResponse parsing not implemented");
        }

        if flags.contains(PacketHeaderFlags::NET_ERROR) {
            header.net_error = Some(NetErrorHeader {});
            bail!("NetError parsing not implemented");
        }

        if flags.contains(PacketHeaderFlags::NET_ERROR_DISCONNECT) {
            header.net_error_disconnect = Some(NetErrorDisconnectHeader {});
            bail!("NetErrorDisconnect parsing not implemented");
        }

        if flags.contains(PacketHeaderFlags::CICMD_COMMAND) {
            header.cicmd_command = Some(CICMDCommandHeader {});
            bail!("CICMDCommand parsing not implemented");
        }

        if flags.contains(PacketHeaderFlags::TIME_SYNC) {
            let time = reader.read_u64()?;
            header.time_sync = Some(TimeSyncHeader { time });
        }

        if flags.contains(PacketHeaderFlags::ECHO_REQUEST) {
            let local_time = reader.read_f32()?;
            header.echo_request = Some(EchoRequestHeader { local_time });
        }

        if flags.contains(PacketHeaderFlags::ECHO_RESPONSE) {
            let local_time = reader.read_f32()?;
            let holding_time = reader.read_f32()?;
            header.echo_response = Some(EchoResponseHeader {
                local_time,
                holding_time,
            });
        }

        if flags.contains(PacketHeaderFlags::FLOW) {
            let data_received = reader.read_u32()?;
            let interval = reader.read_u16()?;
            header.flow = Some(FlowHeader {
                data_received,
                interval,
            });
        }

        Ok(header)
    }
}
