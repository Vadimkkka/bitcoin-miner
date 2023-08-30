use std::env;
use bitcoincore_rpc::RpcApi;
use bitcoincore_rpc::json::{ GetBlockTemplateModes, GetBlockTemplateRules};
use bitcoin::ScriptBuf;

use bitcoin_miner::{connect, mine};
use std::io::{stdout, Write};

fn main() {
    let mut stdout = stdout();

    for i in 0..=100 {
        print!("\rProcessing {0:<1}%...", i);
        stdout.flush().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    /* let rpc = connect();

    let rules = [GetBlockTemplateRules::SegWit];
    let capabilities = [];
    let script_pubkey = env::var("script_pubkey").expect("script_pubkey must be set");
    let script_pubkey_buf = ScriptBuf::from_hex(&script_pubkey).unwrap();
    let message = "aloe-miner"; */

    // loop {
    // let template = rpc.get_block_template(GetBlockTemplateModes::Template, &rules, &capabilities).unwrap();
    // println!("{:#?}", template);
    // mine(&template, &script_pubkey_buf, message);
        // None => println!("Cannot mine block"),
        // Some(block) => {
            // println!("found {:?}", block)
            // let response = rpc.submit_block(&block);
            // println!("response {:?}", response);
        // },
    // };
    // };
}
