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

    let lib_dir = PathBuf::from(env::var("SOLCLIENT_LIB_PATH").unwrap());

    cfg_if::cfg_if! {
        if #[cfg(target_os = "macos")] {
            println!("cargo:rustc-link-search=native={}", lib_dir.join("Darwin/lib").as_path().display());
            println!("cargo:rustc-link-lib=dylib=gssapi_krb5");
            println!("cargo:rustc-link-lib=static=crypto");
            println!("cargo:rustc-link-lib=static=ssl");
            println!("cargo:rustc-link-lib=static=solclient");
        }
        else if #[cfg(target_os = "linux")] {
            println!("cargo:rustc-link-search=native={}", lib_dir.join("Linux/lib").as_path().display());
            println!("cargo:rustc-link-lib=static=crypto");
            println!("cargo:rustc-link-lib=static=ssl");
            println!("cargo:rustc-link-lib=static=solclient");
        }
        else if #[cfg(target_os = "windows")] {
            println!("cargo:rustc-link-search=native={}", lib_dir.as_path().display());
            println!("cargo:rustc-link-search=native={}", lib_dir.join("Win64").display());
            println!("cargo:rustc-link-search=native={}", lib_dir.join("Win64/third-party").display());
            println!("cargo:rustc-link-lib-static=libcrypto_s");
            println!("cargo:rustc-link-lib-static=libssl_s");
        }
    }
}
