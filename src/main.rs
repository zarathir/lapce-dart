use anyhow::{anyhow, Result};
use lapce_plugin::{
    psp_types::{
        lsp_types::{
            request::Initialize, DocumentFilter, DocumentSelector, InitializeParams, MessageType,
            Url,
        },
        Request,
    },
    register_plugin, LapcePlugin, PLUGIN_RPC,
};
use serde_json::Value;

#[derive(Default)]
struct State {}

register_plugin!(State);

fn initialize(params: InitializeParams) -> Result<String> {
    let document_selector: DocumentSelector = vec![DocumentFilter {
        // lsp language id
        language: Some(String::from("dart")),
        // glob pattern
        pattern: Some(String::from("**/*.dart")),
        // like file:
        scheme: None,
    }];
    let mut server_args = vec![];

    if let Some(options) = params.initialization_options.as_ref() {
        if let Some(lsp) = options.get("volt") {
            server_args.push("language-server".to_string());

            if let Some(args) = lsp.get("serverArgs") {
                if let Some(args) = args.as_array() {
                    if !args.is_empty() {
                        server_args = vec![];
                    }
                    for arg in args {
                        if let Some(arg) = arg.as_str() {
                            server_args.push(arg.to_string());
                        }
                    }
                }
            }

            if let Some(server_path) = lsp.get("serverPath") {
                if let Some(server_path) = server_path.as_str() {
                    let server_uri = if !server_path.is_empty() {
                        Url::parse(&format!("urn:{}", server_path))?
                    } else {
                        return Err(anyhow!(
                            "Path to Dart executable not set.\nSet the path in the settings."
                        ));
                    };

                    let uri = server_uri.clone();

                    PLUGIN_RPC.start_lsp(
                        server_uri,
                        server_args,
                        document_selector,
                        params.initialization_options,
                    );

                    return Ok(format!("Trying to start the Dart LSP from\n{}", uri));
                }
            }
        }
    }

    Err(anyhow!("Failed to start the plugin"))
}

impl LapcePlugin for State {
    fn handle_request(&mut self, _id: u64, method: String, params: Value) {
        #[allow(clippy::single_match)]
        match method.as_str() {
            Initialize::METHOD => {
                let params: InitializeParams = serde_json::from_value(params).unwrap();
                match initialize(params) {
                    Ok(msg) => {
                        PLUGIN_RPC.window_log_message(MessageType::INFO, msg.clone());
                        PLUGIN_RPC.window_show_message(MessageType::INFO, msg);
                    }
                    Err(e) => {
                        PLUGIN_RPC.window_log_message(MessageType::ERROR, e.to_string());
                        PLUGIN_RPC.window_show_message(MessageType::ERROR, e.to_string());
                    }
                }
            }
            _ => {}
        }
    }
}
