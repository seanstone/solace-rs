extern crate bindgen;
use std::sync::Arc;
use std::{env, io::Write, path::PathBuf};
use ureq::Agent;

fn build_ureq_agent() -> Agent {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let mut root_store = rustls::RootCertStore::empty();
    for cert in rustls_native_certs::load_native_certs().expect("could not load platform certs") {
        root_store.add(cert).unwrap();
    }
    let tls_config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    ureq::builder().tls_config(Arc::new(tls_config)).build()
}

fn main() {
    // do nothing if we are just building the docs
    if std::env::var("DOCS_RS").is_ok() {
        return;
    }

    cfg_if::cfg_if! {
        if #[cfg(target_os = "macos")] {
            println!("cargo:rustc-link-lib=dylib=gssapi_krb5");
        }
    }

    cfg_if::cfg_if! {
        if #[cfg(target_os = "windows")] {
            println!("cargo:rustc-link-lib-static=libcrypto_s");
            println!("cargo:rustc-link-lib-static=libssl_s");
        } else {
            println!("cargo:rustc-link-lib=static=crypto");
            println!("cargo:rustc-link-lib=static=ssl");
            println!("cargo:rustc-link-lib=static=solclient");
        }
    }
}
