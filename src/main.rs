use std::{fs::File, path::PathBuf};

use anyhow::Result;
use flate2::read::GzDecoder;
use lapce_plugin::{
    psp_types::{
        lsp_types::{request::Initialize, DocumentFilter, InitializeParams, MessageType, Url},
        Request,
    },
    register_plugin, Http, LapcePlugin, PLUGIN_RPC,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default)]
struct State {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    arch: String,
    os: String,
    configuration: Configuration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    language_id: String,
    options: Option<Value>,
}

register_plugin!(State);

fn initialize(params: InitializeParams) -> Result<()> {
    let server_path = params
        .initialization_options
        .as_ref()
        .and_then(|options| options.get("serverPath"))
        .and_then(|server_path| server_path.as_str())
        .and_then(|server_path| {
            if !server_path.is_empty() {
                Some(server_path)
            } else {
                None
            }
        });

    if let Some(server_path) = server_path {
        // TODO: once nightly is released, update plugin api to use request version of start lsp
        // that will inform us of the error
        PLUGIN_RPC.start_lsp(
            Url::parse(&format!("urn:{}", server_path))?,
            Vec::new(),
            vec![DocumentFilter {
                language: Some("elixir".to_string()),
                scheme: None,
                pattern: None,
            }],
            params.initialization_options,
        );
        return Ok(());
    }

    let volt_uri = std::env::var("VOLT_URI")?;
    let server_path = Url::parse(&volt_uri)?.join("elixir-ls")?;
    PLUGIN_RPC.start_lsp(
        server_path,
        Vec::new(),
        vec![DocumentFilter {
            language: Some("elixir".to_string()),
            scheme: None,
            pattern: None,
        }],
        params.initialization_options,
    );
    Ok(())
}

impl LapcePlugin for State {
    fn handle_request(&mut self, _id: u64, method: String, params: Value) {
        #[allow(clippy::single_match)]
        match method.as_str() {
            Initialize::METHOD => {
                let params: InitializeParams = serde_json::from_value(params).unwrap();
                if let Err(e) = initialize(params) {
                    PLUGIN_RPC.stderr(&format!("plugin returned with error: {e}"))
                }
            }
            _ => {}
        }
    }
}
