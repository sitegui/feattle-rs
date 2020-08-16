use feattle::*;
use std::collections::{BTreeMap, BTreeSet};
use strum::VariantNames;

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
    Features {
        extrude_mesh_terrain: bool = true,
        /// A short description
        balance_domestic_coefficients: i32 = 2,
        /// A longer, complete description, bringing attention to contentious issues surrounding
        /// this configuration and what could go wrong if misused.
        invert_career_ladder: f64 = 3.6,
        calculate_money_supply: CalculateMoneySupply = CalculateMoneySupply::High,
        normalize_social_network: String = "normal".to_owned(),
        adjust_emotional_weights: BTreeSet<i32>,
        calibrate_personality_matrix: BTreeSet<CalibratePersonalityMatrix>,
        insert_chaos_generator: BTreeSet<String>,
        map_influence_attributes: BTreeMap<MapInfluenceAttributes, i32>,
        assign_mimic_propagation: BTreeMap<String, i32>,
    }
}

fn main() {
    let features = Features::new();
    dbg!(features.definitions());
}
