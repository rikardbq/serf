pub mod db;
pub mod state;
pub mod util;
pub mod constants;
pub mod error;
pub mod serf_proto {
    include!(concat!(env!("OUT_DIR"), "/serf_proto.rs"));
}