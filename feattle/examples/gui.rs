use feattle::*;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use uuid::Uuid;

feattle_enum! {
    enum CalculateMoneySupply {
        High,
        Low,
    }
}

feattle_enum! {
    enum CalibratePersonalityMatrix {
        Rows,
        Columns,
        Diagonals,
        AntiDiagonals,
    }
}

feattle_enum! {
    enum MapInfluenceAttributes {
        Bias,
        Linear,
        Square,
    }
}

feattles! {
    Features {
        extrude_mesh_terrain: bool,
        assign_mimic_propagation: bool = true,
        balance_domestic_coefficients: u8 = 1,
        invert_career_ladder: i32 = 2,
        concatenate_vertex_nodes: f32 = 3.4,
        normalize_social_network: f64 = 5.6,
        calculate_money_supply: CalculateMoneySupply = CalculateMoneySupply::Low,
        reticulate_splines: Option<Uuid>,
        adjust_emotional_weights: String = "waiting for social validation".to_owned(),
        calibrat_personality_matrix: Vec<i32>,
        inserte_chaos_generator: BTreeSet<String>,
        map_influence_attributes: BTreeMap<Uuid, MapInfluenceAttributes>,
        iterate_chaos_array: Option<String> = Some("Undefined behavior".to_owned()),
        // importe_personality_anchors: i32 = 0,
        // inserte_extension_algorithms: i32 = 0,
        // re_inverte_career_ladder: i32 = 0,
        // aggregate_need_agents: i32 = 0,
        // interprete_family_values: i32 = 0,
        // cabalize_npc_controls: i32 = 0,
        // maximize_social_network: i32 = 0,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    use feattle_sync::persist::Disk;
    use std::sync::Arc;

    let disk_storage = Disk::new("data");
    let features = Arc::new(Features::new(disk_storage));
    tokio::spawn(BackgroundSync::new(&features).run());
    dbg!(features.last_reload());
    dbg!(features.current_values());
    for def in features.definitions() {
        println!("{} = {}", def.key, serde_json::to_string(&def.format)?);
    }

    let panel = AdminPanel::new(features.clone(), "gui".to_owned());
    warp_ui::run_server(panel, ([127, 0, 0, 1], 3030)).await;

    Ok(())
}
