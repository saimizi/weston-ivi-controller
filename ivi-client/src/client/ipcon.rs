use ipcon_sys::{
    ipcon::{Ipcon, IPF_DEFAULT},
    ipcon_msg::{IpconMsg, IpconMsgType},
};

use super::IviClientTransport;
use crate::error::{IviError, Result};
#[allow(unused_imports)]
use jlogger_tracing::{jdebug, jerror, jinfo, jwarn, JloggerBuilder, LevelFilter};
use std::io::{Error as stdIoError, ErrorKind as stdIoErrorKind};
use std::sync::{Arc, Mutex};
use weston_ivi_controller::transport::ipcon::DEFAULT_WESTON_IVI_CONTROLLER_GROUP;
use weston_ivi_controller::transport::ipcon::DEFAULT_WESTON_IVI_CONTROLLER_PEER;

pub struct IpconIviClient {
    // Fields for IP connection client
    ih: Arc<Mutex<Ipcon>>,
}

impl IpconIviClient {
    pub fn ipcon_connect(peer: Option<&str>, server: Option<&str>) -> Result<Self> {
        let ih = Ipcon::new(peer, Some(IPF_DEFAULT))
            .map_err(|e| IviError::ConnectionFailed(format!("IPCON connection failed: {}", e)))?;

        ih.join_group(
            server.unwrap_or(DEFAULT_WESTON_IVI_CONTROLLER_PEER),
            DEFAULT_WESTON_IVI_CONTROLLER_GROUP,
        )
        .map_err(|e| IviError::ConnectionFailed(format!("IPCON connection failed: {}", e)))?;

        Ok(IpconIviClient {
            ih: Arc::new(Mutex::new(ih)),
        })
    }

    pub fn ipcon_disconnect(self) -> Result<()> {
        let ih = self.ih.lock().unwrap();
        ih.leave_group(
            DEFAULT_WESTON_IVI_CONTROLLER_PEER,
            DEFAULT_WESTON_IVI_CONTROLLER_GROUP,
        )
        .map_err(|e| IviError::ConnectionFailed(format!("IPCON disconnection failed: {}", e)))?;
        Ok(())
    }
}

impl IviClientTransport for IpconIviClient {
    fn send_request(&mut self, request: &[u8]) -> crate::Result<()> {
        let ih = self.ih.lock().unwrap();

        ih.send_unicast_msg(DEFAULT_WESTON_IVI_CONTROLLER_PEER, request)
            .map_err(|e| IviError::IoError(stdIoError::new(stdIoErrorKind::Other, e)))?;
        Ok(())
    }

    fn disconnect(&mut self) -> crate::Result<()> {
        let ih = self.ih.lock().unwrap();
        ih.leave_group(
            DEFAULT_WESTON_IVI_CONTROLLER_PEER,
            DEFAULT_WESTON_IVI_CONTROLLER_GROUP,
        )
        .map_err(|e| IviError::ConnectionFailed(format!("IPCON disconnection failed: {}", e)))?;
        Ok(())
    }

    fn receive_response(&mut self) -> crate::Result<Vec<u8>> {
        let ih = self.ih.lock().unwrap();

        loop {
            let msg = ih
                .receive_msg()
                .map_err(|e| IviError::IoError(stdIoError::new(stdIoErrorKind::Other, e)))?;

            if let IpconMsg::IpconMsgUser(body) = msg {
                if body.msg_type == IpconMsgType::IpconMsgTypeNormal
                    && body.peer == DEFAULT_WESTON_IVI_CONTROLLER_PEER
                {
                    return Ok(body.buf);
                }
            }
        }
    }
}
