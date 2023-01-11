use anyhow::Result;
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

fn initialize(params: InitializeParams) -> Result<()> {
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
        server_args.push("language-server".to_string());
        if let Some(args) = options.get("serverArgs") {
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

        let search_bin = match std::env::var("VOLT_OS").as_deref() {
            Ok("windows") => "where",
            _ => "which",
        };

        let server_path = options
            .get("serverPath")
            .and_then(|server_path| server_path.as_str())
            .and_then(|server_path| {
                if !server_path.is_empty() {
                    Some(server_path)
                } else {
                    None
                }
            });

        if let Some(server_path) = server_path {
            let found_dart = PLUGIN_RPC
                .execute_process(search_bin.to_string(), vec![server_path.to_string()])
                .map(|r| r.success)
                .unwrap_or(false);

            if found_dart {
                PLUGIN_RPC.start_lsp(
                    Url::parse(&format!("urn:{server_path}"))?,
                    server_args,
                    document_selector,
                    params.initialization_options,
                );
            } else {
                PLUGIN_RPC.window_show_message(
                    MessageType::ERROR,
                    "Unable to find dart executable.\nPlease enter a valid path.".to_string(),
                );
            }
        } else {
            PLUGIN_RPC.window_show_message(
                MessageType::ERROR,
                "Path to Dart executable not set.\nPlease enter a valid path.".to_string(),
            );
        }
    }

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
