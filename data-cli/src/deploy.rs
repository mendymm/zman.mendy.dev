use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;

use sha2::{Digest, Sha256};
use ureq::unversioned::multipart::{Form, Part};
use zip::ZipArchive;

#[derive(Debug, Clone, serde::Deserialize)]
struct DeploymentResponse {
    result: Deployment,
    success: bool,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct Deployment {
    id: String,
    url: String,
    short_id: String,
    environment: String,
    created_on: String,
}

pub fn deploy(bundle_path: &str, account_id: &str, token: &str, project: &str) {
    println!("Reading {}...", bundle_path);

    let file = File::open(bundle_path).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();

    println!("Hashing files:");
    let mut manifest: BTreeMap<String, String> = BTreeMap::new();
    let mut headers_content: Option<String> = None;
    let mut redirects_content: Option<String> = None;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let name = file.name().to_string();

        if file.is_dir() {
            continue;
        }

        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();

        if data.is_empty() {
            continue;
        }

        let hash = hex::encode(Sha256::digest(&data));
        println!("  {} -> {}", name, &hash[..8]);
        manifest.insert(name.clone(), hash);

        if name == "_headers" {
            headers_content = Some(String::from_utf8(data.clone()).unwrap());
        }

        if name == "_redirects" {
            redirects_content = Some(String::from_utf8(data.clone()).unwrap());
        }
    }

    let manifest_json = serde_json::to_string(&manifest).unwrap();
    println!("\nManifest: {} files", manifest.len());

    if headers_content.is_some() {
        println!("Found _headers file in bundle");
    }

    if redirects_content.is_some() {
        println!("Found _redirects file in bundle");
    }

    let url = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/pages/projects/{}/deployments",
        account_id, project
    );

    println!("Uploading to Cloudflare Pages...");

    let bundle_data = std::fs::read(bundle_path).unwrap();
    let file_part = Part::bytes(&bundle_data).file_name("bundle.zip");

    let mut form = Form::new()
        .part("file", file_part)
        .text("manifest", &manifest_json);

    if let Some(headers) = &headers_content {
        let headers_part = Part::bytes(headers.as_bytes()).file_name("_headers");
        form = form.part("_headers", headers_part);
    }

    if let Some(redirects) = &redirects_content {
        let redirects_part = Part::bytes(redirects.as_bytes()).file_name("_redirects");
        form = form.part("_redirects", redirects_part);
    }

    let response = ureq::post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send(form)
        .unwrap();

    let response_text = response.into_body().read_to_string().unwrap();

    let deployment: DeploymentResponse = serde_json::from_str(&response_text).unwrap();

    if deployment.success {
        println!("✓ Deployed successfully!");
        println!("  URL: {}", deployment.result.url);
        println!("  ID: {}", deployment.result.id);
        println!("  Short ID: {}", deployment.result.short_id);
        println!("  Environment: {}", deployment.result.environment);
        println!("  Created: {}", deployment.result.created_on);
    } else {
        eprintln!("Deployment failed");
        eprintln!("Response: {}", response_text);
    }
}
