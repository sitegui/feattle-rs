mod reflection;
mod try_from_value;

use crossbeam_utils::atomic::AtomicCell;
use parking_lot::{RwLock, RwLockReadGuard};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use strum_macros::EnumString;
use try_from_value::TryFromValue;
use uuid::Uuid;

macro_rules! define_enum {
    ($key:ident { $($variant:ident),* $(,)? }) => {
        #[derive(Debug, Clone, Copy, Eq, PartialEq, EnumString, PartialOrd, Ord)]
        enum $key { $($variant),* }

        impl TryFromValue for $key {
            fn try_from_value(value: Value) -> Option<Self> {
                value.as_str().and_then(|s| s.parse().ok())
            }
        }
    }
}

macro_rules! read_copy {
    ($key:ident, $kind:ty) => {
        pub fn $key(&self) -> $kind {
            self.$key.load()
        }
    };
}

macro_rules! read_non_copy {
    ($key:ident, $kind:ty) => {
        pub fn $key(&self) -> RwLockReadGuard<$kind> {
            self.$key.read()
        }
    };
}

macro_rules! write_copy {
    ($self:ident, $values:ident, $key:ident, $kind:ty) => {
        if let Some(value) = $values.remove(stringify!($key)) {
            match <$kind as TryFromValue>::try_from_value(value) {
                Some(x) => $self.$key.store(x),
                None => log::error!("Failed to parse {}", stringify!($key)),
            }
        }
    };
}

macro_rules! write_non_copy {
    ($self:ident, $values:ident, $key:ident, $kind:ty) => {
        if let Some(value) = $values.remove(stringify!($key)) {
            match <$kind as TryFromValue>::try_from_value(value) {
                Some(x) => *$self.$key.write() = x,
                None => log::error!("Failed to parse {}", stringify!($key)),
            }
        }
    };
}

define_enum! {
    CalculateMoneySupply {
        High,
        Low,
    }
}

define_enum! {
    CalibratePersonalityMatrix {
        Rows,
        Columns,
        Diagonals,
        AntiDiagonals,
    }
}

define_enum! {
    MapInfluenceAttributes {
        Bias,
        Linear,
        Square,
    }
}

struct InternalStorage {
    extrude_mesh_terrain: AtomicCell<bool>,
    balance_domestic_coefficients: AtomicCell<i32>,
    invert_career_ladder: AtomicCell<f64>,
    calculate_money_supply: AtomicCell<CalculateMoneySupply>,
    reticulate_splines: AtomicCell<Uuid>,
    normalize_social_network: RwLock<String>,
    adjust_emotional_weights: RwLock<BTreeSet<i32>>,
    calibrate_personality_matrix: RwLock<BTreeSet<CalibratePersonalityMatrix>>,
    concatenate_vertex_nodes: RwLock<BTreeSet<Uuid>>,
    insert_chaos_generator: RwLock<BTreeSet<String>>,
    map_influence_attributes: RwLock<BTreeMap<MapInfluenceAttributes, i32>>,
    iterate_chaos_array: RwLock<BTreeMap<Uuid, i32>>,
    assign_mimic_propagation: RwLock<BTreeMap<String, i32>>,
}

impl InternalStorage {
    read_copy! { extrude_mesh_terrain, bool }
    read_copy! { balance_domestic_coefficients, i32 }
    read_copy! { invert_career_ladder, f64 }
    read_copy! { calculate_money_supply, CalculateMoneySupply }
    read_copy! { reticulate_splines, Uuid }
    read_non_copy! { normalize_social_network, String }
    read_non_copy! { adjust_emotional_weights, BTreeSet<i32> }
    read_non_copy! { calibrate_personality_matrix, BTreeSet<CalibratePersonalityMatrix> }
    read_non_copy! { concatenate_vertex_nodes, BTreeSet<Uuid> }
    read_non_copy! { insert_chaos_generator, BTreeSet<String> }
    read_non_copy! { map_influence_attributes, BTreeMap<MapInfluenceAttributes, i32> }
    read_non_copy! { iterate_chaos_array, BTreeMap<Uuid, i32> }
    read_non_copy! { assign_mimic_propagation, BTreeMap<String, i32> }

    fn update(&self, mut values: BTreeMap<String, Value>) {
        write_copy!(self, values, extrude_mesh_terrain, bool);
        write_copy!(self, values, balance_domestic_coefficients, i32);
        write_copy!(self, values, invert_career_ladder, f64);
        write_copy!(self, values, calculate_money_supply, CalculateMoneySupply);
        write_copy!(self, values, reticulate_splines, Uuid);
        write_non_copy!(self, values, normalize_social_network, String);
        write_non_copy!(self, values, adjust_emotional_weights, BTreeSet<i32>);
        write_non_copy!(
            self,
            values,
            calibrate_personality_matrix,
            BTreeSet<CalibratePersonalityMatrix>
        );
        write_non_copy!(self, values, concatenate_vertex_nodes, BTreeSet<Uuid>);
        write_non_copy!(self, values, insert_chaos_generator, BTreeSet<String>);
        write_non_copy!(self, values, map_influence_attributes, BTreeMap<MapInfluenceAttributes, i32>);
        write_non_copy!(self, values, iterate_chaos_array, BTreeMap<Uuid, i32>);
        write_non_copy!(self, values, assign_mimic_propagation, BTreeMap<String, i32>);
    }
}

fn main() {
    println!("Hello, world!");
}
