extern crate crossbeam_channel;
extern crate reqwest;
use mech_core::{hash_string, TableIndex, Table, Value, ValueType, ValueMethods, Transaction, Change, TableId, Register};
use mech_utilities::{Machine, MachineRegistrar, RunLoopMessage};
//use std::sync::mpsc::{self, Sender};
use std::thread::{self};
use crossbeam_channel::Sender;
use std::collections::HashMap;

lazy_static! {
  static ref HTTP_REQUEST: u64 = hash_string("http/request");
  static ref URI: u64 = hash_string("uri");
  static ref HEADER: u64 = hash_string("header");
  static ref RESPONSE: u64 = hash_string("response");
}

export_machine!(http_request, http_request_reg);

extern "C" fn http_request_reg(registrar: &mut dyn MachineRegistrar, outgoing: Sender<RunLoopMessage>) -> String {
  registrar.register_machine(Box::new(Request{outgoing}));
  "#http/request = [|uri header response|]".to_string()
}

#[derive(Debug)]
pub struct Request {
  outgoing: Sender<RunLoopMessage>,
}

impl Machine for Request {

  fn name(&self) -> String {
    "http/request".to_string()
  }

  fn id(&self) -> u64 {
    Register{table_id: TableId::Global(*HTTP_REQUEST), row: TableIndex::All, column: TableIndex::All}.hash()
  }

  fn on_change(&mut self, table: &Table) -> Result<(), String> {
    println!("HTTPREQUEST CHANGED");
    println!("{:?}", table.rows);
    for i in 1..=table.rows {
      let uri = table.get_string(&TableIndex::Index(i), &TableIndex::Alias(*URI));
      match uri {
        Some((uri,_)) => {
          let row = TableIndex::Index(i);
          let outgoing = self.outgoing.clone();
          let uri = uri.clone();
          let request_handle = thread::spawn(move || {
            match reqwest::blocking::get(uri) {
              Ok(response) => {
                if response.status().is_success() {
                  let text = response.text().unwrap();
                  println!("SENDING A TXN");
                  println!("Got TExt {:?}", text);
                  outgoing.send(RunLoopMessage::Transaction(Transaction{changes: vec![
                    Change::Set{table_id: *HTTP_REQUEST, values: vec![(row, TableIndex::Alias(*RESPONSE), Value::from_string(&text))]},
                    Change::InternString{string: text},
                  ]}));
                } else if response.status().is_server_error() {
                  // TODO Handle Error
                } else {
                  // TODO Handle Error
                }
              }
              Err(_) => (), // TODO Handle errors
            }
          });
        }
        _ => (), // TODO Send error
      }
    }
    Ok(())
  }
}