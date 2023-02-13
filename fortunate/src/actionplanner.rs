use rand::{Rng, thread_rng};

use crate::cursor::BitArraySignalKeyCursor;
use crate::event::EventType;
use crate::primitives::{Pair, dunwrap_s};
use crate::{algorithms, primitives::TString};
use crate::fnode::get_node_s_signals;
use crate::dynamoc::{get_dynamo_client};
use aws_sdk_dynamodb::{ Client, };
use crate::node::NodeSignalKeyRefSer;

pub struct ActionPlanner {
  status: std::string::String,
  dynamo_client: Client,
}

pub struct ExpMap {
  pub e2: u64, // a
  pub e10: u64, // b
}

impl ExpMap {
  pub fn new(e2: u64, e10: u64) -> Self {
    ExpMap { e2: e2, e10: e10 } 
  }
}

pub trait IActionPlan<T> {
  fn act(&self) -> T;
  fn divisor(&self) -> u64;
  fn signals(&self) -> Vec<NodeSignalKeyRefSer>;
  fn event_type(&self) -> EventType;
}

pub struct BitArrayActionPlan<T> {
  pub signal_key_ref_pairs: Vec<NodeSignalKeyRefSer>,
  pub result: Option<T>,
}

impl IActionPlan<bool> for BitArrayActionPlan<bool> {

  fn signals(&self) -> Vec<NodeSignalKeyRefSer> {
    let mut v: Vec<NodeSignalKeyRefSer> = vec![];

    for it in self.signal_key_ref_pairs.iter() {
      v.push(it.to_owned());
    };

    v
  }

  fn divisor(&self) -> u64 {
    let mut return_value = 1;

    for kp in self.signal_key_ref_pairs.iter() {
      let p = kp.refindex();

      let mut e = 0;
      let mut x = 0;

      if (p < 10) {
        e = 2;
        x = p;
      }
      else {
        e = 10;
        x = p - 10;
      }
      return_value = return_value * u64::pow(e, (x+1).try_into().unwrap());
    };
    return_value
  }

  fn act(&self) -> bool {
    let mut return_value = true;
    
    for kp in self.signal_key_ref_pairs.iter() {
      let k = &kp.signal_key;
      let p = kp.refindex();

      let c = BitArraySignalKeyCursor::new(&k);
      return_value = c.bit(p.try_into().unwrap()) && return_value;
    };

    return_value
  }

  fn event_type(&self) -> EventType {
    EventType::PE("00".to_string())
  }


}

impl ActionPlanner {

  pub async fn new() -> Self {
    ActionPlanner { status: "".to_string(), dynamo_client: get_dynamo_client().await }
  }

  pub async fn get_actionplan_for_event_pe<V>(
    &self, 
    epoch: &std::string::String, expmap: &ExpMap) -> BitArrayActionPlan<V> {
    let signals 
        = get_node_s_signals(&self.dynamo_client, epoch).await;

    let estr = self.range_expmap(expmap);

    let mut c = crate::cursor::Cursor::new(&estr);

    let mut signal_idx = 0;
    let mut key_ref_pairs: Vec<NodeSignalKeyRefSer> = vec![];

    loop {
      let s = c.advance_until_changed();
      let exp = s.len();
      let signal = signals.get(signal_idx).unwrap();

      let refsignal = NodeSignalKeyRefSer { 
        signal_key: dunwrap_s(signal.get("signal_key").unwrap())          
      };

      key_ref_pairs.push(refsignal);

      let mut v_exp = 0;

      if ( s.chars().nth(0).unwrap() == 'a') {
        v_exp = exp;
      }
      else if (s.chars().nth(0).unwrap() == 'b') {
        v_exp = exp-1+10;
      }

      signal_idx = (signal_idx + 1) % signals.len();

      if (c.eof()) {
        break;
      }
    }

    BitArrayActionPlan::<V> { signal_key_ref_pairs: key_ref_pairs, result: None }
  }

  fn range_expmap(&self, expmap: &ExpMap) -> String {
    let a = "a".repeat(expmap.e2.try_into().unwrap()) + &"b".repeat(expmap.e10.try_into().unwrap());
    a.shuffle_n(3)
  }

}


