use deno_bindgen::deno_bindgen;
use std::str;
use libc::{sysconf, _SC_CHILD_MAX};
use std::io::{Read, Write};
use std::net::TcpStream;
use sysinfo::{Pid, PidExt, ProcessExt, System, SystemExt};
use native_tls::TlsConnector;
use openssl::x509::X509;
use openssl::pkey::PKey;
use openssl::sign::Verifier;
use openssl::hash::MessageDigest;

//
// PROCESS COUNT
//
#[deno_bindgen]
fn get_max_children() -> i64 {
    let max_processes = unsafe { sysconf(_SC_CHILD_MAX) };
    return max_processes;
}
//
// PROCESS INFO
//
#[deno_bindgen]
pub struct ProcessInfo {
    cpu_usage: f32,
    start_time: u64,
    run_time: u64,
    virtual_memory: u64,
    memory: u64,
}
#[deno_bindgen]
pub fn get_process_info(pid: u32) -> ProcessInfo {
    let mut system = System::new_all();
    system.refresh_process(Pid::from_u32(pid));
    system.refresh_cpu();
    if let Some(process) = system.process(Pid::from_u32(pid)) {
        ProcessInfo {
            cpu_usage: process.cpu_usage(),
            start_time: process.start_time(),
            run_time: process.run_time(),
            virtual_memory: process.virtual_memory(),
            memory: process.memory(),
        }
    } else {
        ProcessInfo {
            cpu_usage: 0.0,
            start_time: 0,
            run_time: 0,
            virtual_memory: 0,
            memory: 0,
        }
    }
}
//
// PROCESS TIME
//
#[deno_bindgen]
pub struct ProcessTime {
    user_time: u64,
    system_time: u64,
}
#[deno_bindgen]
pub fn get_process_time(pid: u32) -> ProcessTime {
    match psutil::process::Process::new(pid) {
        Ok(process) => match process.cpu_times() {
            Ok(times) => {
                let user_time = times.user();
                let system_time = times.system();
                ProcessTime {
                    user_time: user_time.as_millis() as u64,
                    system_time: system_time.as_millis() as u64,
                }
            }
            Err(_) => ProcessTime {
                user_time: 0,
                system_time: 0,
            },
        },
        Err(_) => ProcessTime {
            user_time: 0,
            system_time: 0,
        },
    }
}
//
// SSL CERT
//
#[deno_bindgen]
pub struct CertDetails {
    certificat: String,
    public_key: String,
    error: String
}
#[deno_bindgen(non_blocking)]
fn get_cert(url: &str, port: u32, ignore_error: u8) -> CertDetails {
    let mut builder = TlsConnector::builder();
    if ignore_error == 1 {
        builder.danger_accept_invalid_certs(true);
    }
    let connector = match builder.build() {
        Ok(c) => c,
        Err(e) => {
            return CertDetails {
                certificat: String::new(),
                public_key: String::new(),
                error: e.to_string()
            }
        }
    };
    let stream = match TcpStream::connect(format!("{}:{}", url, port)) {
        Ok(s) => s,
        Err(e) => {
            return CertDetails {
                certificat: String::new(),
                public_key: String::new(),
                error: e.to_string()
            }
        }
    };
    let mut stream = match connector.connect(url, stream) {
        Ok(s) => s,
        Err(e) => {
            return CertDetails {
                certificat: String::new(),
                public_key: String::new(),
                error: e.to_string()
            }
        }
    };
    let _stream_res = match stream.write_all(b"HEAD / HTTP/1.0\r\n\r\n") {
        Ok(s) => Some(s),
        Err(_e) => None,
    };
    let mut res = vec![];
    let _read_err = match stream.read_to_end(&mut res){
        Ok(s) => Some(s),
        Err(_e) => None,
    };
    let certificate = match stream.peer_certificate() {
        Ok(Some(cert)) => cert,
        Ok(None) => {
            return CertDetails {
                certificat: String::new(),
                public_key: String::new(),
                error: "NO_CERT".to_string(),
            }
        },
        Err(e) => {
            return CertDetails {
                certificat: String::new(),
                public_key: String::new(),
                error: e.to_string(),
            }
        }
    };
    let cert = match X509::from_der(&certificate.to_der().unwrap_or_else(|_| return vec![])) {
        Ok(x) => x,
        Err(e) => {
            return CertDetails {
                certificat: String::new(),
                public_key: String::new(),
                error: e.to_string(),
            }
        }
    };
    let cert_pem = cert.to_pem().unwrap_or_else(|_| return vec![]);
    let public_key = if let Ok(pk) = cert.public_key() {
        pk
    } else {
        return CertDetails {
            certificat: String::new(),
            public_key: String::new(),
            error: "NO_PUBLIC_KEY".to_string()
        };
    };
    let public_key_pem = public_key
        .public_key_to_pem()
        .unwrap_or_else(|_| return vec![]);
    let cert_string = String::from_utf8(cert_pem).unwrap_or_else(|_| String::new());
    let public_key_string = String::from_utf8(public_key_pem).unwrap_or_else(|_| String::new());
    CertDetails {
        certificat: cert_string,
        public_key: public_key_string,
        error: "".to_string()
    }
}

#[deno_bindgen(non_blocking)]
pub fn verify_signature(pub_key_pem: &str, message: &[u8], signature: &[u8]) -> String {
    let pub_key = match PKey::public_key_from_pem(pub_key_pem.as_bytes()) {
        Ok(x) => x,
        Err(e) => {
            return e.to_string()
        }
    };
    let mut verifier = match Verifier::new(MessageDigest::sha256(), &pub_key) {
        Ok(x) => x,
        Err(e) => {
            return e.to_string()
        }
    };
    let _res1 = match verifier.update(message) {
        Ok(x) => x,
        Err(e) => {
            return e.to_string()
        }
    };
    let res2 = match verifier.verify(signature) {
        Ok(x) => x,
        Err(e) => {
            return e.to_string()
        }
    };
    if res2 == true {
        return "OK".to_string()
    }
    return "NOK".to_string()
}
//
// OLD FUNCTION
//
#[deno_bindgen]
fn get_cpu_usage(pid: u32) -> f32 {
    let mut system = System::new_all();
    system.refresh_process(Pid::from_u32(pid));
    system.refresh_cpu();
    if let Some(process) = system.process(Pid::from_u32(pid)) {
        return process.cpu_usage();
    } else {
        return -1.0;
    }
}
#[deno_bindgen]
fn get_start_time(pid: u32) -> u64 {
    let mut system = System::new_all();
    system.refresh_process(Pid::from_u32(pid));
    if let Some(process) = system.process(Pid::from_u32(pid)) {
        return process.start_time();
    } else {
        return 0xFFFFFFFFFFFFFFFF;
    }
}
#[deno_bindgen]
fn get_run_time(pid: u32) -> u64 {
    let mut system = System::new_all();
    system.refresh_process(Pid::from_u32(pid));
    if let Some(process) = system.process(Pid::from_u32(pid)) {
        return process.run_time();
    } else {
        return 0xFFFFFFFFFFFFFFFF;
    }
}
#[deno_bindgen]
fn get_virtual_memory(pid: u32) -> u64 {
    let mut system = System::new_all();
    system.refresh_process(Pid::from_u32(pid));
    if let Some(process) = system.process(Pid::from_u32(pid)) {
        return process.virtual_memory();
    } else {
        return 0xFFFFFFFFFFFFFFFF;
    }
}
#[deno_bindgen]
fn get_memory(pid: u32) -> u64 {
    let mut system = System::new_all();
    system.refresh_process(Pid::from_u32(pid));
    if let Some(process) = system.process(Pid::from_u32(pid)) {
        return process.memory();
    } else {
        return 0xFFFFFFFFFFFFFFFF;
    }
}
#[deno_bindgen]
fn get_cpu_stime(pid: u32) -> u64 {
    if let Ok(process) = psutil::process::Process::new(pid) {
        if let Ok(cpu_time) = process.cpu_times() {
            let stime = cpu_time.system();
            //let utime = cpu_time.user;
            let value: u128 = stime.as_millis();
            //let value_string = value.to_string();
            //let c_string = std::ffi::CString::new(value_string).expect("Failed to create CString");
            //c_string.into_raw()
            return value as u64;
        } else {
            return 0xFFFFFFFFFFFFFFFF;
        }
    } else {
        return 0xFFFFFFFFFFFFFFFF;
    }
}
#[deno_bindgen]
fn get_cpu_utime(pid: u32) -> u64 {
    if let Ok(process) = psutil::process::Process::new(pid) {
        if let Ok(cpu_time) = process.cpu_times() {
            let utime = cpu_time.user();
            let value: u128 = utime.as_millis();
            //let value_string = value.to_string();
            //let c_string = std::ffi::CString::new(value_string).expect("Failed to create CString");
            //c_string.into_raw()
            return value as u64;
        } else {
            return 0xFFFFFFFFFFFFFFFF;
        }
    } else {
        return 0xFFFFFFFFFFFFFFFF;
    }
}
