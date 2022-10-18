use anyhow::Result;
use lapce_plugin::{
    psp_types::{
        lsp_types::{
            request::Initialize, DocumentFilter, DocumentSelector, InitializeParams,
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
        if let Some(lsp) = options.get("volt") {
            server_args.push(format!("language-server"));

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
                        Url::parse(&format!("urn:dart"))?
                    };

                    PLUGIN_RPC.start_lsp(
                        server_uri,
                        server_args,
                        document_selector,
                        params.initialization_options,
                    );
                    return Ok(());
                }
            }
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
