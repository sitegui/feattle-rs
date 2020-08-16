use feattle::*;
use std::collections::{BTreeMap, BTreeSet};
use strum::VariantNames;
use uuid::Uuid;

feattle_enum! {
    CalculateMoneySupply {
        High,
        Low,
    }
}

feattle_enum! {
    CalibratePersonalityMatrix {
        Rows,
        Columns,
        Diagonals,
        AntiDiagonals,
    }
}

feattle_enum! {
    MapInfluenceAttributes {
        Bias,
        Linear,
        Square,
    }
}

feattles! {
    InternalStorage {
        extrude_mesh_terrain: bool,
        balance_domestic_coefficients: i32,
        invert_career_ladder: f64,
        calculate_money_supply: CalculateMoneySupply,
        reticulate_splines: Uuid,
        normalize_social_network: String,
        adjust_emotional_weights: BTreeSet<i32>,
        calibrate_personality_matrix: BTreeSet<CalibratePersonalityMatrix>,
        concatenate_vertex_nodes: BTreeSet<Uuid>,
        insert_chaos_generator: BTreeSet<String>,
        map_influence_attributes: BTreeMap<MapInfluenceAttributes, i32>,
        iterate_chaos_array: BTreeMap<Uuid, i32>,
        assign_mimic_propagation: BTreeMap<String, i32>,
    }
}

fn main() {
    println!("Hello, world!");
}
