mod feattle_value;
mod reflection;

use feattle_value::{FeattleStringValue, FeattleValue};
use parking_lot::{RwLock, RwLockReadGuard};
use reflection::StringFormat;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use strum::VariantNames;
use strum_macros::{EnumString, EnumVariantNames};
use uuid::Uuid;

macro_rules! define_enum {
    ($key:ident { $($variant:ident),* $(,)? }) => {
        #[derive(Debug, Clone, Copy, Eq, PartialEq, EnumString, PartialOrd, Ord, EnumVariantNames)]
        enum $key { $($variant),* }

        impl FeattleStringValue for $key {
            fn serialized_string_format() -> StringFormat {
                StringFormat::Choices(&$key::VARIANTS)
            }
        }
    }
}

macro_rules! read {
    ($key:ident, $kind:ty) => {
        pub fn $key(&self) -> RwLockReadGuard<$kind> {
            self.$key.read()
        }
    };
}

macro_rules! write {
    ($self:ident, $values:ident, $key:ident, $kind:ty) => {
        if let Some(value) = $values.remove(stringify!($key)) {
            match <$kind as FeattleValue>::try_from_json(value) {
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
    extrude_mesh_terrain: RwLock<bool>,
    balance_domestic_coefficients: RwLock<i32>,
    invert_career_ladder: RwLock<f64>,
    calculate_money_supply: RwLock<CalculateMoneySupply>,
    reticulate_splines: RwLock<Uuid>,
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
    read! { extrude_mesh_terrain, bool }
    read! { balance_domestic_coefficients, i32 }
    read! { invert_career_ladder, f64 }
    read! { calculate_money_supply, CalculateMoneySupply }
    read! { reticulate_splines, Uuid }
    read! { normalize_social_network, String }
    read! { adjust_emotional_weights, BTreeSet<i32> }
    read! { calibrate_personality_matrix, BTreeSet<CalibratePersonalityMatrix> }
    read! { concatenate_vertex_nodes, BTreeSet<Uuid> }
    read! { insert_chaos_generator, BTreeSet<String> }
    read! { map_influence_attributes, BTreeMap<MapInfluenceAttributes, i32> }
    read! { iterate_chaos_array, BTreeMap<Uuid, i32> }
    read! { assign_mimic_propagation, BTreeMap<String, i32> }

    fn update(&self, mut values: BTreeMap<String, Value>) {
        write!(self, values, extrude_mesh_terrain, bool);
        write!(self, values, balance_domestic_coefficients, i32);
        write!(self, values, invert_career_ladder, f64);
        write!(self, values, calculate_money_supply, CalculateMoneySupply);
        write!(self, values, reticulate_splines, Uuid);
        write!(self, values, normalize_social_network, String);
        write!(self, values, adjust_emotional_weights, BTreeSet<i32>);
        write!(
            self,
            values,
            calibrate_personality_matrix,
            BTreeSet<CalibratePersonalityMatrix>
        );
        write!(self, values, concatenate_vertex_nodes, BTreeSet<Uuid>);
        write!(self, values, insert_chaos_generator, BTreeSet<String>);
        write!(self, values, map_influence_attributes, BTreeMap<MapInfluenceAttributes, i32>);
        write!(self, values, iterate_chaos_array, BTreeMap<Uuid, i32>);
        write!(self, values, assign_mimic_propagation, BTreeMap<String, i32>);
    }
}

fn main() {
    println!("Hello, world!");
}
