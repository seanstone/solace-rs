extern crate bindgen;
use std::sync::Arc;
use std::{env, io::Write, path::PathBuf, path::Path};
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

fn download_and_unpack(url: &str, tarball_path: PathBuf, tarball_unpack_path: PathBuf) {
    let mut content = Vec::new();
    build_ureq_agent()
        .get(url)
        .call()
        .unwrap()
        .into_reader()
        .read_to_end(&mut content)
        .unwrap();

    let mut file_gz = std::fs::File::create(tarball_path.clone()).unwrap();
    file_gz.write_all(&content).unwrap();
    file_gz.sync_data().unwrap();

    let file_gz = std::fs::File::open(tarball_path).unwrap();
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(file_gz));
    archive
        .entries()
        .unwrap()
        .filter_map(|r| r.ok())
        .map(|mut entry| -> std::io::Result<PathBuf> {
            let strip_path = entry.path()?.iter().skip(1).collect::<std::path::PathBuf>();
            let path = tarball_unpack_path.join(strip_path);
            entry.unpack(&path)?;
            Ok(path)
        })
        .filter_map(|e| e.ok())
        .for_each(|x| println!("> {}", x.display()));
}

fn handle_platform(solclient_tarball_url: &str, solclient_tarball_path: &Path, solclient_folder_path: &Path) {
    if !solclient_folder_path.is_dir() {
        eprintln!(
            "Solclient not found. Downloading from {}",
            solclient_tarball_url
        );
        download_and_unpack(
            solclient_tarball_url,
            solclient_tarball_path.to_path_buf(),
            solclient_folder_path.to_path_buf(),
        );
    }
    let lib_dir = solclient_folder_path.join("lib");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=solclient");
}

fn main() {
    // do nothing if we are just building the docs
    if std::env::var("DOCS_RS").is_ok() {
        return;
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let solclient_folder_path = out_dir.join("solclient");
    let solclient_tarball_path = out_dir.join("solclient.tar.gz");

    cfg_if::cfg_if! {
        if #[cfg(target_os = "macos")] {
            handle_platform(
                "https://products.solace.com/download/C_API_OSX",
                &solclient_tarball_path,
                &solclient_folder_path,
            );
        } else if #[cfg(target_os = "linux")] {
            handle_platform(
                "https://products.solace.com/download/C_API_LINUX64",
                &solclient_tarball_path,
                &solclient_folder_path,
            );
        } else if #[cfg(target_os = "ios")] {
            handle_platform(
                "https://products.solace.com/download/C_API_IOS",
                &solclient_tarball_path,
                &solclient_folder_path,
            );
        }
    }
}
